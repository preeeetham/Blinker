//! Rebuilds analytics for blinks the event log doesn't know about.
//!
//! Postgres is the source of truth for blinks and confirmed payments; the
//! event log is the source of truth for everything else (views, action
//! POSTs). If the log is lost — a wiped disk, or a host with ephemeral
//! storage — blinks would otherwise become invisible to the analytics
//! engine and their new events would be counted as orphans.
//!
//! So on startup, once the log replay has caught up, any blink in Postgres
//! that the engine hasn't seen is re-announced through the normal pipeline
//! (`Created`, `Activated`, one `Paid` per recorded payment) with its
//! original timestamps. The events are written to the log like any others,
//! so this runs at most once per lost log.

use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

use blinker_analytics::{Asset, BlinkId, EventKind};
use blinker_solana::Pubkey;

use crate::{db, AppState};

pub async fn run(st: &AppState) -> anyhow::Result<()> {
    // Wait for the consumer to finish replaying what's already in the log.
    let target = st.metrics.committed_offset.load(Relaxed);
    for _ in 0..600 {
        if st.metrics.consumer_offset.load(Relaxed) >= target {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let mut blinks = 0;
    let mut payments = 0;
    for row in db::all_blinks(&st.db).await? {
        let Some(id) = BlinkId::parse(&row.id) else { continue };
        if st.engine.read().unwrap().contains(&id) {
            continue;
        }
        let asset = if row.is_sol() { Asset::Sol } else { Asset::Spl };
        st.bus.emit_at(row.created_at.timestamp_millis() as u64, id, EventKind::Created { asset }).await;
        if let Some(at) = row.activated_at {
            let fee = row.fee_lamports.max(0) as u64;
            st.bus.emit_at(at.timestamp_millis() as u64, id, EventKind::Activated { fee_lamports: fee }).await;
        }
        for p in db::payments_for(&st.db, &row.id).await? {
            let Ok(payer) = p.payer.parse::<Pubkey>() else { continue };
            let kind = EventKind::Paid { amount: p.amount.max(0) as u64, payer: payer.0 };
            st.bus.emit_at(p.created_at.timestamp_millis() as u64, id, kind).await;
            payments += 1;
        }
        blinks += 1;
    }
    if blinks > 0 {
        tracing::info!(blinks, payments, "rebuilt analytics for blinks missing from the event log");
    }
    Ok(())
}
