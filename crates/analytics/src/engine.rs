use std::collections::HashMap;

use serde::Serialize;

use crate::event::{Asset, BlinkId, CodecError, Event, EventKind};
use crate::hll::HyperLogLog;
use crate::ring::{Bucket, Ring};

const MINUTE: u64 = 60_000;
const HOUR: u64 = 60 * MINUTE;
const DAY: u64 = 24 * HOUR;

struct BlinkStats {
    asset: Asset,
    created_ms: u64,
    activated: bool,
    views: u64,
    posts: u64,
    paid: u64,
    volume: u128,
    payers: HyperLogLog,
    minutes: Ring<60>,
    hours: Ring<168>,
}

impl BlinkStats {
    fn new(asset: Asset, ts: u64) -> Self {
        Self {
            asset,
            created_ms: ts,
            activated: false,
            views: 0,
            posts: 0,
            paid: 0,
            volume: 0,
            payers: HyperLogLog::default(),
            minutes: Ring::new(MINUTE),
            hours: Ring::new(HOUR),
        }
    }
}

#[derive(Default)]
struct Global {
    by_kind: [u64; 5],
    sol_volume: u128,
    payers: HyperLogLog,
    seconds: Option<Box<Ring<300>>>,
    minutes: Option<Box<Ring<60>>>,
}

/// The analytics state machine. Single-writer: the pipeline's consumer
/// task owns it and readers take snapshots through a lock.
pub struct Engine {
    blinks: HashMap<BlinkId, Box<BlinkStats>>,
    global: Global,
    applied: u64,
    decode_errors: u64,
    orphans: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BlinkSummary {
    pub id: BlinkId,
    pub asset: Asset,
    pub created_ms: u64,
    pub activated: bool,
    pub views: u64,
    pub posts: u64,
    pub paid: u64,
    /// Base units (lamports for SOL blinks). String: may exceed JS's 2^53.
    pub volume: String,
    pub unique_payers: u64,
    /// paid / views, 0..=1.
    pub conversion: f64,
    pub views_24h: u64,
    pub paid_24h: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SeriesPoint {
    pub t: u64,
    pub views: u32,
    pub posts: u32,
    pub paid: u32,
    pub volume: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct GlobalSummary {
    pub blinks: usize,
    pub active_blinks: usize,
    pub views: u64,
    pub posts: u64,
    pub paid: u64,
    pub sol_volume_lamports: String,
    pub unique_payers: u64,
    pub events_applied: u64,
    pub decode_errors: u64,
    pub events_per_sec_1m: f64,
    pub engine_memory_bytes: usize,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        let global = Global {
            seconds: Some(Box::new(Ring::new(1000))),
            minutes: Some(Box::new(Ring::new(MINUTE))),
            ..Default::default()
        };
        Self { blinks: HashMap::with_capacity(1024), global, applied: 0, decode_errors: 0, orphans: 0 }
    }

    /// Decodes and applies one log record. Undecodable records are counted
    /// and skipped — a bad record must never stall the consumer.
    pub fn apply_bytes(&mut self, record: &[u8]) -> Result<(), CodecError> {
        match Event::decode(record) {
            Ok(e) => {
                self.apply(&e);
                Ok(())
            }
            Err(err) => {
                self.decode_errors += 1;
                Err(err)
            }
        }
    }

    pub fn apply(&mut self, e: &Event) {
        self.applied += 1;
        let g = &mut self.global;
        let tag = match e.kind {
            EventKind::Created { .. } => 0,
            EventKind::Activated { .. } => 1,
            EventKind::View => 2,
            EventKind::ActionPost { .. } => 3,
            EventKind::Paid { .. } => 4,
        };
        g.by_kind[tag] += 1;
        if let Some(b) = g.seconds.as_mut().and_then(|r| r.slot(e.ts_ms)) {
            b.views += 1; // "views" lane of the global per-second ring = all events
        }

        if let EventKind::Created { asset } = e.kind {
            self.blinks.entry(e.blink).or_insert_with(|| Box::new(BlinkStats::new(asset, e.ts_ms)));
            return;
        }
        let Some(s) = self.blinks.get_mut(&e.blink) else {
            // Event for a blink we never saw created (e.g. log retention
            // dropped its Created record). Count it globally only.
            self.orphans += 1;
            return;
        };

        let mut delta = Bucket::default();
        match e.kind {
            EventKind::Created { .. } => unreachable!(),
            EventKind::Activated { .. } => s.activated = true,
            EventKind::View => {
                s.views += 1;
                delta.views = 1;
            }
            EventKind::ActionPost { .. } => {
                s.posts += 1;
                delta.posts = 1;
            }
            EventKind::Paid { amount, payer } => {
                s.paid += 1;
                s.volume += amount as u128;
                s.payers.insert_key(&payer);
                delta.paid = 1;
                delta.volume = amount;
                g.payers.insert_key(&payer);
                if s.asset == Asset::Sol {
                    g.sol_volume += amount as u128;
                }
            }
        }
        if let Some(b) = s.minutes.slot(e.ts_ms) {
            b.add(&delta);
        }
        if let Some(b) = s.hours.slot(e.ts_ms) {
            b.add(&delta);
        }
        if let Some(b) = g.minutes.as_mut().and_then(|r| r.slot(e.ts_ms)) {
            b.add(&delta);
        }
    }

    pub fn summary(&self, id: &BlinkId, now_ms: u64) -> Option<BlinkSummary> {
        let s = self.blinks.get(id)?;
        let day = s.hours.sum_since(now_ms, DAY);
        Some(BlinkSummary {
            id: *id,
            asset: s.asset,
            created_ms: s.created_ms,
            activated: s.activated,
            views: s.views,
            posts: s.posts,
            paid: s.paid,
            volume: s.volume.to_string(),
            unique_payers: if s.paid == 0 { 0 } else { s.payers.estimate().clamp(1, s.paid) },
            conversion: if s.views == 0 { 0.0 } else { (s.paid as f64 / s.views as f64).min(1.0) },
            views_24h: day.views as u64,
            paid_24h: day.paid as u64,
        })
    }

    /// Per-minute series for the last hour, or per-hour for the last week.
    pub fn series(&self, id: &BlinkId, now_ms: u64, hourly: bool) -> Option<Vec<SeriesPoint>> {
        let s = self.blinks.get(id)?;
        let raw = if hourly { s.hours.series(now_ms) } else { s.minutes.series(now_ms) };
        Some(raw.into_iter().map(to_point).collect())
    }

    /// Global events/second for the last five minutes (all event kinds).
    pub fn throughput(&self, now_ms: u64) -> Vec<(u64, u32)> {
        self.global.seconds.as_ref().map(|r| r.series(now_ms).into_iter().map(|(t, b)| (t, b.views)).collect()).unwrap_or_default()
    }

    pub fn global_series(&self, now_ms: u64) -> Vec<SeriesPoint> {
        self.global.minutes.as_ref().map(|r| r.series(now_ms).into_iter().map(to_point).collect()).unwrap_or_default()
    }

    pub fn global(&self, now_ms: u64) -> GlobalSummary {
        let g = &self.global;
        let last_min = g.seconds.as_ref().map(|r| r.sum_since(now_ms, MINUTE).views).unwrap_or(0);
        GlobalSummary {
            blinks: self.blinks.len(),
            active_blinks: self.blinks.values().filter(|s| s.activated).count(),
            views: g.by_kind[2],
            posts: g.by_kind[3],
            paid: g.by_kind[4],
            sol_volume_lamports: g.sol_volume.to_string(),
            unique_payers: if g.by_kind[4] == 0 { 0 } else { g.payers.estimate().clamp(1, g.by_kind[4]) },
            events_applied: self.applied,
            decode_errors: self.decode_errors,
            events_per_sec_1m: last_min as f64 / 60.0,
            engine_memory_bytes: self.memory_bytes(),
        }
    }

    /// Top blinks by confirmed payments, then views.
    pub fn top(&self, n: usize, now_ms: u64) -> Vec<BlinkSummary> {
        let mut ids: Vec<(&BlinkId, (u64, u64))> =
            self.blinks.iter().filter(|(_, s)| s.activated).map(|(id, s)| (id, (s.paid, s.views))).collect();
        // Partial selection: O(n) instead of a full sort for large maps.
        let k = n.min(ids.len());
        if k == 0 {
            return vec![];
        }
        ids.select_nth_unstable_by(k - 1, |a, b| b.1.cmp(&a.1));
        ids.truncate(k);
        ids.sort_unstable_by_key(|a| std::cmp::Reverse(a.1));
        ids.into_iter().filter_map(|(id, _)| self.summary(id, now_ms)).collect()
    }

    /// Whether the engine has seen this blink's `Created` event.
    pub fn contains(&self, id: &BlinkId) -> bool {
        self.blinks.contains_key(id)
    }

    pub fn events_applied(&self) -> u64 {
        self.applied
    }

    pub fn orphans(&self) -> u64 {
        self.orphans
    }

    /// Approximate heap footprint: per-blink boxes plus map overhead.
    pub fn memory_bytes(&self) -> usize {
        let per_blink = std::mem::size_of::<BlinkStats>() + HyperLogLog::memory_bytes();
        let map = self.blinks.capacity()
            * (std::mem::size_of::<BlinkId>() + std::mem::size_of::<Box<BlinkStats>>() + 1);
        self.blinks.len() * per_blink + map + std::mem::size_of::<Ring<300>>() + std::mem::size_of::<Ring<60>>() + 2 * HyperLogLog::memory_bytes()
    }
}

fn to_point((t, b): (u64, Bucket)) -> SeriesPoint {
    SeriesPoint { t, views: b.views, posts: b.posts, paid: b.paid, volume: b.volume }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> BlinkId {
        BlinkId::parse(s).unwrap()
    }

    #[test]
    fn folds_events() {
        let mut e = Engine::new();
        let b = id("blink0000001");
        let t = 10 * DAY;
        e.apply(&Event { ts_ms: t, blink: b, kind: EventKind::Created { asset: Asset::Sol } });
        e.apply(&Event { ts_ms: t, blink: b, kind: EventKind::Activated { fee_lamports: 1_000_000 } });
        for i in 0..10u8 {
            e.apply(&Event { ts_ms: t + i as u64 * 1000, blink: b, kind: EventKind::View });
        }
        for i in 0..4u8 {
            let payer = [i; 32];
            e.apply(&Event { ts_ms: t + 5_000, blink: b, kind: EventKind::Paid { amount: 100_000_000, payer } });
        }
        // Repeat payer.
        e.apply(&Event { ts_ms: t + 6_000, blink: b, kind: EventKind::Paid { amount: 1, payer: [0; 32] } });

        let s = e.summary(&b, t + 10_000).unwrap();
        assert!(s.activated);
        assert_eq!((s.views, s.paid), (10, 5));
        assert_eq!(s.volume, "400000001");
        assert_eq!(s.unique_payers, 4);
        assert_eq!(s.views_24h, 10);
        let g = e.global(t + 10_000);
        assert_eq!(g.sol_volume_lamports, "400000001");
        assert_eq!(e.top(5, t).len(), 1);
        let series = e.series(&b, t + 10_000, false).unwrap();
        assert_eq!(series.len(), 60);
        assert_eq!(series.iter().map(|p| p.views).sum::<u32>(), 10);
    }

    #[test]
    fn orphans_and_garbage_are_counted() {
        let mut e = Engine::new();
        e.apply(&Event { ts_ms: 1, blink: id("nobody000000"), kind: EventKind::View });
        assert_eq!(e.orphans(), 1);
        assert!(e.apply_bytes(&[1, 2, 3]).is_err());
        assert_eq!(e.global(1).decode_errors, 1);
    }
}
