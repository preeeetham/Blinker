//! On-chain watcher: a resilient `logsSubscribe` websocket client.
//!
//! Every Blinker transaction (fees and payments) mentions the treasury, so
//! one subscription sees them all. The loop:
//!
//! 1. connect, subscribe `{ mentions: [treasury] }` at `confirmed`
//! 2. backfill with `getSignaturesForAddress` — anything that landed while
//!    we were disconnected is caught here (the verifier dedups overlap)
//! 3. forward each notification's signature to the verifier
//! 4. ping every 20s; treat 45s of silence as a dead connection
//! 5. on any error: exponential backoff with full jitter (0.5s → 30s cap),
//!    reset once a connection has stayed healthy for a minute

use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use blinker_solana::Pubkey;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;

use crate::metrics::Metrics;
use crate::rpc::Rpc;
use crate::verify::Verifier;

pub fn spawn(ws_url: String, treasury: Pubkey, rpc: Rpc, verifier: Verifier, metrics: Arc<Metrics>) {
    tokio::spawn(async move {
        let mut backoff = Duration::from_millis(500);
        loop {
            let started = Instant::now();
            let res = session(&ws_url, &treasury, &rpc, &verifier, &metrics).await;
            metrics.ws_connected.store(0, Relaxed);
            Metrics::inc(&metrics.c.ws_disconnects);
            if started.elapsed() > Duration::from_secs(60) {
                backoff = Duration::from_millis(500);
            }
            let sleep = Duration::from_millis(rand::random::<u64>() % (backoff.as_millis() as u64 + 1));
            tracing::warn!(error = %res.err().map(|e| format!("{e:#}")).unwrap_or_default(), retry_in_ms = sleep.as_millis() as u64, "watcher disconnected");
            tokio::time::sleep(sleep).await;
            backoff = (backoff * 2).min(Duration::from_secs(30));
        }
    });
}

async fn session(url: &str, treasury: &Pubkey, rpc: &Rpc, verifier: &Verifier, m: &Metrics) -> Result<()> {
    let (mut ws, _) = tokio::time::timeout(Duration::from_secs(10), tokio_tungstenite::connect_async(url))
        .await
        .map_err(|_| anyhow!("connect timeout"))??;
    let sub = json!({
        "jsonrpc": "2.0", "id": 1, "method": "logsSubscribe",
        "params": [{ "mentions": [treasury.to_string()] }, { "commitment": "confirmed" }]
    });
    ws.send(Message::Text(sub.to_string().into())).await?;
    m.ws_connected.store(1, Relaxed);
    Metrics::inc(&m.c.ws_connects);
    tracing::info!(%url, "watcher subscribed to treasury logs");

    // Backfill whatever we missed while disconnected.
    match rpc.signatures_for_address(treasury, 100).await {
        Ok(sigs) => {
            for s in sigs.iter().rev() {
                verifier.submit(s);
            }
        }
        Err(e) => tracing::warn!("backfill failed: {e:#}"),
    }

    let mut ping = tokio::time::interval(Duration::from_secs(20));
    ping.tick().await;
    let mut last_rx = Instant::now();
    loop {
        tokio::select! {
            msg = ws.next() => {
                let msg = msg.ok_or_else(|| anyhow!("stream closed"))??;
                last_rx = Instant::now();
                match msg {
                    Message::Text(t) => {
                        if let Some(sig) = notification_signature(&t) {
                            Metrics::inc(&m.c.ws_notifications);
                            verifier.submit(&sig);
                        }
                    }
                    Message::Ping(p) => ws.send(Message::Pong(p)).await?,
                    Message::Close(f) => return Err(anyhow!("server closed: {f:?}")),
                    _ => {}
                }
            }
            _ = ping.tick() => {
                if last_rx.elapsed() > Duration::from_secs(45) {
                    return Err(anyhow!("no traffic for 45s"));
                }
                ws.send(Message::Ping(Vec::new().into())).await?;
            }
        }
    }
}

/// Extracts the signature from a successful `logsNotification`.
pub fn notification_signature(text: &str) -> Option<String> {
    let v: Value = serde_json::from_str(text).ok()?;
    if v.get("method")?.as_str()? != "logsNotification" {
        return None;
    }
    let value = v.pointer("/params/result/value")?;
    if !value.get("err").is_none_or(Value::is_null) {
        return None;
    }
    Some(value.get("signature")?.as_str()?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_notifications() {
        let ok = r#"{"jsonrpc":"2.0","method":"logsNotification","params":{"result":{"context":{"slot":5},"value":{"signature":"5h6x","err":null,"logs":[]}},"subscription":0}}"#;
        assert_eq!(notification_signature(ok).as_deref(), Some("5h6x"));
        let failed = ok.replace(r#""err":null"#, r#""err":{"InstructionError":[0,"Custom"]}"#);
        assert_eq!(notification_signature(&failed), None);
        assert_eq!(notification_signature(r#"{"jsonrpc":"2.0","result":7,"id":1}"#), None);
        assert_eq!(notification_signature("garbage"), None);
    }
}
