//! Blinker's analytics engine.
//!
//! Events are small fixed-layout binary records (see [`event`]) appended to
//! the event log. This crate folds them into per-blink statistics whose
//! memory is fixed per blink no matter how much traffic it gets:
//!
//! * totals (views, action POSTs, confirmed payments, volume)
//! * a 60 × 1-minute ring and a 168 × 1-hour ring for charts
//! * a HyperLogLog sketch (1 KiB) for unique payers, instead of an
//!   unbounded `HashSet<Pubkey>`
//!
//! Ingest is allocation-free once a blink's stats exist.

pub mod engine;
pub mod event;
pub mod hll;
pub mod ring;

pub use engine::{BlinkSummary, Engine, GlobalSummary, SeriesPoint};
pub use event::{Asset, BlinkId, Event, EventKind};
