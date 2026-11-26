//! Process resource usage straight from the OS.
//!
//! * `getrusage(2)` for CPU time and peak RSS (portable POSIX).
//! * On Linux, `/proc/self/status` for current RSS / thread count and
//!   `/proc/self/fd` for open file descriptors.
//! * On macOS, `proc_pidinfo`'s task info isn't in libc's safe surface, so
//!   current RSS falls back to peak RSS.
//!
//! A sampler task computes CPU% from rusage deltas once per second.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, Default)]
pub struct ProcSnapshot {
    pub pid: u32,
    pub cpu_user_ms: u64,
    pub cpu_sys_ms: u64,
    /// CPU utilisation over the last sample interval (100 = one full core).
    pub cpu_percent: f64,
    pub rss_bytes: u64,
    pub peak_rss_bytes: u64,
    pub threads: u64,
    pub open_fds: u64,
    pub voluntary_ctx_switches: u64,
    pub involuntary_ctx_switches: u64,
    pub uptime_secs: u64,
    pub os: &'static str,
}

fn rusage() -> libc::rusage {
    // SAFETY: getrusage only writes into the zeroed struct we pass.
    unsafe {
        let mut ru: libc::rusage = std::mem::zeroed();
        libc::getrusage(libc::RUSAGE_SELF, &mut ru);
        ru
    }
}

fn tv_ms(tv: libc::timeval) -> u64 {
    tv.tv_sec as u64 * 1000 + tv.tv_usec as u64 / 1000
}

#[cfg(target_os = "linux")]
fn linux_status() -> (Option<u64>, Option<u64>) {
    let Ok(s) = std::fs::read_to_string("/proc/self/status") else { return (None, None) };
    let field = |name: &str| {
        s.lines()
            .find_map(|l| l.strip_prefix(name))
            .and_then(|v| v.split_whitespace().next()?.parse::<u64>().ok())
    };
    (field("VmRSS:").map(|kb| kb * 1024), field("Threads:"))
}

fn open_fds() -> u64 {
    #[cfg(target_os = "linux")]
    let dir = "/proc/self/fd";
    #[cfg(not(target_os = "linux"))]
    let dir = "/dev/fd";
    std::fs::read_dir(dir).map(|d| d.count() as u64).unwrap_or(0)
}

pub struct ProcSampler {
    started: Instant,
    last: Mutex<(Instant, u64, ProcSnapshot)>,
}

impl Default for ProcSampler {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcSampler {
    pub fn new() -> Self {
        let now = Instant::now();
        Self { started: now, last: Mutex::new((now, 0, ProcSnapshot::default())) }
    }

    /// Takes a fresh sample and updates CPU%. Call periodically.
    pub fn sample(&self) -> ProcSnapshot {
        let ru = rusage();
        let user = tv_ms(ru.ru_utime);
        let sys = tv_ms(ru.ru_stime);
        // ru_maxrss is KiB on Linux, bytes on macOS.
        #[cfg(target_os = "macos")]
        let peak = ru.ru_maxrss as u64;
        #[cfg(not(target_os = "macos"))]
        let peak = ru.ru_maxrss as u64 * 1024;

        #[cfg(target_os = "linux")]
        let (rss, threads) = {
            let (r, t) = linux_status();
            (r.unwrap_or(peak), t.unwrap_or(0))
        };
        #[cfg(not(target_os = "linux"))]
        let (rss, threads) = (peak, 0u64);

        let now = Instant::now();
        let mut last = self.last.lock().unwrap();
        let cpu_total = user + sys;
        let dt = now.duration_since(last.0).as_millis().max(1) as f64;
        let cpu_percent = (cpu_total.saturating_sub(last.1)) as f64 / dt * 100.0;
        let snap = ProcSnapshot {
            pid: std::process::id(),
            cpu_user_ms: user,
            cpu_sys_ms: sys,
            cpu_percent: (cpu_percent * 10.0).round() / 10.0,
            rss_bytes: rss,
            peak_rss_bytes: peak,
            threads,
            open_fds: open_fds(),
            voluntary_ctx_switches: ru.ru_nvcsw as u64,
            involuntary_ctx_switches: ru.ru_nivcsw as u64,
            uptime_secs: self.started.elapsed().as_secs(),
            os: std::env::consts::OS,
        };
        *last = (now, cpu_total, snap);
        snap
    }

    /// Most recent sample without touching the OS.
    pub fn latest(&self) -> ProcSnapshot {
        let mut s = self.last.lock().unwrap().2;
        s.uptime_secs = self.started.elapsed().as_secs();
        s
    }

    pub fn spawn(self: &std::sync::Arc<Self>) {
        let me = self.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(1));
            loop {
                tick.tick().await;
                me.sample();
            }
        });
    }
}
