//! Live view of the backend's internals: process resources from
//! getrusage//proc, the event pipeline, the commit log and latency
//! histograms. Polls `/api/system` once a second.

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json::Value;

use crate::api;
use crate::fx::{Bars, GridBg, Sparkline, SpotCard};
use crate::util::{fmt_bytes, fmt_num, fmt_us, poll};

#[component]
pub fn SystemPage() -> impl IntoView {
    let sys = RwSignal::new(Value::Null);
    let cpu_hist = RwSignal::new(Vec::<f64>::new());
    let rss_hist = RwSignal::new(Vec::<f64>::new());
    poll(1000, move || {
        spawn_local(async move {
            if let Ok(v) = api::get::<Value>("/api/system").await {
                let cpu = v["process"]["cpu_percent"].as_f64().unwrap_or(0.0);
                let rss = v["process"]["rss_bytes"].as_f64().unwrap_or(0.0);
                cpu_hist.update(|h| {
                    h.push(cpu);
                    if h.len() > 60 {
                        h.remove(0);
                    }
                });
                rss_hist.update(|h| {
                    h.push(rss);
                    if h.len() > 60 {
                        h.remove(0);
                    }
                });
                sys.set(v);
            }
        })
    });

    let n = move |path: &str| sys.with(|v| v.pointer(path).and_then(Value::as_f64).unwrap_or(0.0));
    let throughput = Signal::derive(move || {
        sys.with(|v| {
            v["throughput"]
                .as_array()
                .map(|a| a.iter().map(|p| (p[1].as_f64().unwrap_or(0.0), 0.0)).collect::<Vec<_>>())
                .unwrap_or_default()
        })
    });
    let q_rows = ["http", "action_post", "rpc", "log_commit", "verify"];
    let counter_rows = [
        ("events_emitted", "events emitted"),
        ("events_committed", "events committed"),
        ("events_dropped", "events shed (backpressure)"),
        ("log_batches", "group commits"),
        ("action_gets", "action GETs"),
        ("action_posts", "action POSTs"),
        ("ws_notifications", "ws notifications"),
        ("ws_disconnects", "ws reconnects"),
        ("verify_ok", "verified"),
        ("verify_dedup", "deduplicated"),
        ("verify_retries", "verify retries"),
        ("rpc_calls", "rpc calls"),
        ("rpc_retries", "rpc retries"),
        ("blockhash_refreshes", "blockhash refreshes"),
    ];

    view! {
        <section class="page system">
            <GridBg />
            <div class="page-head row-between">
                <div>
                    <span class="eyebrow"><span class="dot live"></span>"Live · 1s refresh"</span>
                    <h1 class="h1">"System telemetry"</h1>
                    <p class="muted">"Straight from the running process: getrusage(2), /proc, the event pipeline and the commit log."</p>
                </div>
                <a class="btn ghost" href="/metrics" target="_blank">"Prometheus /metrics ↗"</a>
            </div>

            <div class="stat-grid">
                <SpotCard class="stat">
                    <span class="stat-label">"CPU"</span>
                    <span class="stat-value">{move || format!("{:.1}%", n("/process/cpu_percent"))}</span>
                    <Sparkline values=cpu_hist width=200.0 height=36.0 class="spark mini" />
                </SpotCard>
                <SpotCard class="stat">
                    <span class="stat-label">{move || if sys.with(|v| v["process"]["os"] == "linux") { "RSS" } else { "Peak RSS" }}</span>
                    <span class="stat-value">{move || fmt_bytes(n("/process/rss_bytes"))}</span>
                    <Sparkline values=rss_hist width=200.0 height=36.0 class="spark mini" />
                </SpotCard>
                <SpotCard class="stat">
                    <span class="stat-label">"Events / sec (1m)"</span>
                    <span class="stat-value">{move || fmt_num((throughput.get().iter().rev().take(60).map(|x| x.0).sum::<f64>() / 60.0 * 10.0).round() / 10.0)}</span>
                </SpotCard>
                <SpotCard class="stat">
                    <span class="stat-label">"Watcher"</span>
                    <span class="stat-value">
                        {move || if sys.with(|v| v["watcher"]["connected"] == true) { "● live" } else { "○ reconnecting" }}
                    </span>
                </SpotCard>
            </div>

            <div class="sys-grid">
                <div class="panel span2">
                    <div class="panel-head">
                        <h3>"Pipeline throughput (events/s, last 5 min)"</h3>
                    </div>
                    <Bars values=throughput width=900.0 height=140.0 />
                </div>

                <div class="panel">
                    <h3>"Event pipeline"</h3>
                    <div class="gauge">
                        <div class="gauge-fill" style:width=move || format!("{:.1}%", n("/pipeline/queue_depth") / n("/pipeline/queue_capacity").max(1.0) * 100.0)></div>
                    </div>
                    <p class="muted small">{move || format!("queue {}/{}", n("/pipeline/queue_depth"), n("/pipeline/queue_capacity"))}</p>
                    <ul class="spec-list mono">
                        <li><span>"committed offset"</span><b>{move || fmt_num(n("/pipeline/committed_offset"))}</b></li>
                        <li><span>"consumer offset"</span><b>{move || fmt_num(n("/pipeline/consumer_offset"))}</b></li>
                        <li><span>"consumer lag"</span><b>{move || n("/pipeline/consumer_lag")}</b></li>
                        <li><span>"last / max batch"</span><b>{move || format!("{} / {}", n("/pipeline/last_batch"), n("/pipeline/max_batch"))}</b></li>
                    </ul>
                </div>

                <div class="panel">
                    <h3>"Commit log"</h3>
                    <ul class="spec-list mono">
                        <li><span>"segments"</span><b>{move || n("/log/segments")}</b></li>
                        <li><span>"size"</span><b>{move || fmt_bytes(n("/log/bytes"))}</b></li>
                        <li><span>"records"</span><b>{move || fmt_num(n("/log/next_offset") - n("/log/first_offset"))}</b></li>
                        <li><span>"fsyncs"</span><b>{move || fmt_num(n("/log/fsyncs"))}</b></li>
                        <li><span>"last fdatasync"</span><b>{move || fmt_us(n("/log/last_fsync_us"))}</b></li>
                        <li><span>"torn bytes recovered"</span><b>{move || n("/log/recovered_truncated_bytes")}</b></li>
                    </ul>
                </div>

                <div class="panel">
                    <h3>"Process"</h3>
                    <ul class="spec-list mono">
                        <li><span>"pid"</span><b>{move || n("/process/pid")}</b></li>
                        <li><span>"uptime"</span><b>{move || format!("{}s", n("/process/uptime_secs"))}</b></li>
                        <li><span>"cpu user / sys"</span><b>{move || format!("{}ms / {}ms", n("/process/cpu_user_ms"), n("/process/cpu_sys_ms"))}</b></li>
                        <li><span>"threads"</span><b>{move || { let t = n("/process/threads"); if t == 0.0 { "n/a (linux only)".to_string() } else { t.to_string() } }}</b></li>
                        <li><span>"open fds"</span><b>{move || n("/process/open_fds")}</b></li>
                        <li><span>"ctx switches v / iv"</span><b>{move || format!("{} / {}", fmt_num(n("/process/voluntary_ctx_switches")), fmt_num(n("/process/involuntary_ctx_switches")))}</b></li>
                        <li><span>"analytics memory"</span><b>{move || fmt_bytes(n("/engine/memory_bytes"))}</b></li>
                    </ul>
                </div>

                <div class="panel span2">
                    <h3>"Latency (log2 histogram buckets)"</h3>
                    <table class="table mono">
                        <thead>
                            <tr><th>"path"</th><th>"count"</th><th>"mean"</th><th>"p50 ≤"</th><th>"p90 ≤"</th><th>"p99 ≤"</th></tr>
                        </thead>
                        <tbody>
                            {q_rows
                                .iter()
                                .map(|k| {
                                    let k = *k;
                                    view! {
                                        <tr>
                                            <td>{k}</td>
                                            <td>{move || fmt_num(n(&format!("/latency/{k}/count")))}</td>
                                            <td>{move || fmt_us(n(&format!("/latency/{k}/mean_us")))}</td>
                                            <td>{move || fmt_us(n(&format!("/latency/{k}/p50_us")))}</td>
                                            <td>{move || fmt_us(n(&format!("/latency/{k}/p90_us")))}</td>
                                            <td>{move || fmt_us(n(&format!("/latency/{k}/p99_us")))}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()}
                        </tbody>
                    </table>
                </div>

                <div class="panel">
                    <h3>"Counters"</h3>
                    <ul class="spec-list mono">
                        {counter_rows
                            .iter()
                            .map(|(k, label)| {
                                let k = *k;
                                view! { <li><span>{*label}</span><b>{move || fmt_num(n(&format!("/counters/{k}")))}</b></li> }
                            })
                            .collect_view()}
                    </ul>
                </div>
            </div>
        </section>
    }
}
