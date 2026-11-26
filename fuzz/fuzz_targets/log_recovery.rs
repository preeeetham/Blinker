//! Crash recovery: whatever garbage is left at the end of the active
//! segment, `Log::open` must not panic, must keep every record that was
//! committed before the garbage, and must accept new appends afterwards.
#![no_main]
use blinker_eventlog::{Config, Log, SyncPolicy};
use libfuzzer_sys::fuzz_target;
use std::io::Write;

fuzz_target!(|data: &[u8]| {
    let dir = tempfile::tempdir().unwrap();
    let cfg = Config { segment_bytes: 1 << 20, sync: SyncPolicy::Never, retention_bytes: None };
    let n = (data.first().copied().unwrap_or(0) % 16) as u64;
    {
        let mut log = Log::open(dir.path(), cfg.clone()).unwrap();
        for i in 0..n {
            log.append(&i.to_le_bytes()).unwrap();
        }
        log.commit().unwrap();
    }
    let seg = dir.path().join(format!("{:020}.log", 0));
    std::fs::OpenOptions::new().append(true).open(&seg).unwrap().write_all(data).unwrap();

    let mut log = Log::open(dir.path(), cfg).unwrap();
    assert!(log.next_offset() >= n, "lost committed records");
    let mut r = log.reader(0).unwrap();
    for i in 0..n {
        let (off, payload) = r.next().unwrap().expect("committed record missing");
        assert_eq!(off, i);
        assert_eq!(payload, i.to_le_bytes());
    }
    log.append(b"after-recovery").unwrap();
    log.commit().unwrap();
});
