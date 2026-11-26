//! Payment verifier: turns a transaction signature into a database update and
//! an analytics event, exactly once.
//!
//! Signatures arrive from three places — the websocket watcher, the
//! creator's confirm call, and the Actions `next` callback — so the same
//! signature routinely shows up 2–3 times. Dedup happens at three levels:
//! an in-flight/seen set in memory, `UPDATE … WHERE status='pending'` for
//! fees, and the `payments.signature` primary key for payments.

use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use blinker_analytics::{BlinkId, EventKind};
use blinker_solana::parse::{actions, resolve_keys, Action};
use blinker_solana::{Pubkey, VersionedTransaction};
use sqlx::PgPool;
use tokio::sync::{mpsc, Semaphore};

use crate::db;
use crate::metrics::Metrics;
use crate::pipeline::EventBus;
use crate::rpc::{FetchedTx, Rpc};
use crate::txbuild::{parse_memo, AtaCache, Tag};

const MAX_ATTEMPTS: u32 = 8;
const SEEN_CAPACITY: usize = 16_384;

/// What a transaction claims to be, before checking it against the DB.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finding {
    Fee { blink: BlinkId, lamports_to_treasury: u64 },
    Payment { blink: BlinkId, payer: Pubkey, sol: Vec<(Pubkey, u64)>, tokens: Vec<(Option<Pubkey>, Pubkey, u64)> },
}

/// Pure inspection of a decoded transaction (fuzzed and unit-tested).
pub fn inspect(tx: &VersionedTransaction, keys: &[Pubkey], treasury: &Pubkey) -> Option<Finding> {
    let msg = tx.message.inner();
    let payer = *keys.first()?;
    let mut tag = None;
    let mut sol = Vec::new();
    let mut tokens = Vec::new();
    for a in actions(msg, keys) {
        match a {
            Action::Memo(m) => {
                if tag.is_none() {
                    tag = parse_memo(m);
                }
            }
            Action::SolTransfer { to, lamports, .. } => sol.push((to, lamports)),
            Action::TokenTransfer { mint, to, amount, .. } => tokens.push((mint, to, amount)),
        }
    }
    match tag? {
        Tag::Fee(blink) => Some(Finding::Fee {
            blink,
            lamports_to_treasury: sol.iter().filter(|(to, _)| to == treasury).map(|(_, l)| *l).fold(0u64, u64::saturating_add),
        }),
        Tag::Pay(blink) => Some(Finding::Payment { blink, payer, sol, tokens }),
    }
}

/// Amount a payment finding delivered to a blink's recipient.
pub fn delivered(f: &Finding, blink: &db::BlinkRow, atas: &AtaCache) -> Option<u64> {
    let Finding::Payment { sol, tokens, .. } = f else { return None };
    let recipient: Pubkey = blink.recipient.parse().ok()?;
    let total = if blink.is_sol() {
        sol.iter().filter(|(to, _)| *to == recipient).map(|(_, l)| *l).fold(0u64, u64::saturating_add)
    } else {
        let mint: Pubkey = blink.mint.as_deref()?.parse().ok()?;
        let tp: Pubkey = blink.token_program.as_deref()?.parse().ok()?;
        let ata = atas.get(&recipient, &mint, &tp);
        tokens
            .iter()
            .filter(|(m, to, _)| *to == ata && m.is_none_or(|m| m == mint))
            .map(|(_, _, a)| *a)
            .fold(0u64, u64::saturating_add)
    };
    (total > 0).then_some(total)
}

/// Bounded FIFO set of signatures we've finished with.
struct Seen {
    set: HashSet<String>,
    order: VecDeque<String>,
    inflight: HashSet<String>,
}

impl Seen {
    fn insert_done(&mut self, s: String) {
        self.inflight.remove(&s);
        if self.set.insert(s.clone()) {
            self.order.push_back(s);
            if self.order.len() > SEEN_CAPACITY
                && let Some(old) = self.order.pop_front()
            {
                self.set.remove(&old);
            }
        }
    }
}

struct Job {
    sig: String,
    attempt: u32,
    enqueued: Instant,
}

#[derive(Clone)]
pub struct Verifier {
    tx: mpsc::Sender<Job>,
    seen: Arc<Mutex<Seen>>,
    metrics: Arc<Metrics>,
}

pub fn is_signature(s: &str) -> bool {
    (64..=88).contains(&s.len()) && bs58::decode(s).into_vec().is_ok_and(|v| v.len() == 64)
}

#[derive(Clone)]
struct Ctx {
    rpc: Rpc,
    db: PgPool,
    bus: EventBus,
    treasury: Pubkey,
    atas: Arc<AtaCache>,
    metrics: Arc<Metrics>,
}

impl Verifier {
    pub fn spawn(
        rpc: Rpc,
        db: PgPool,
        bus: EventBus,
        treasury: Pubkey,
        workers: usize,
        atas: Arc<AtaCache>,
        metrics: Arc<Metrics>,
    ) -> Self {
        let (tx, mut rx) = mpsc::channel::<Job>(4096);
        let seen = Arc::new(Mutex::new(Seen { set: HashSet::new(), order: VecDeque::new(), inflight: HashSet::new() }));
        let me = Verifier { tx: tx.clone(), seen: seen.clone(), metrics: metrics.clone() };
        let ctx = Ctx { rpc, db, bus, treasury, atas, metrics };
        let permits = Arc::new(Semaphore::new(workers.max(1)));
        let requeue = me.clone();
        tokio::spawn(async move {
            while let Some(job) = rx.recv().await {
                let permit = permits.clone().acquire_owned().await.expect("semaphore");
                let ctx = ctx.clone();
                let me = requeue.clone();
                tokio::spawn(async move {
                    let _permit = permit;
                    me.run(&ctx, job).await;
                });
            }
        });
        me
    }

    /// Queues a signature for verification. False if malformed or the
    /// queue is full (caller can tell the client to retry).
    pub fn submit(&self, sig: &str) -> bool {
        if !is_signature(sig) {
            return false;
        }
        {
            let mut s = self.seen.lock().unwrap();
            if s.set.contains(sig) || !s.inflight.insert(sig.to_string()) {
                Metrics::inc(&self.metrics.c.verify_dedup);
                return true;
            }
        }
        let ok = self.tx.try_send(Job { sig: sig.to_string(), attempt: 0, enqueued: Instant::now() }).is_ok();
        if !ok {
            self.seen.lock().unwrap().inflight.remove(sig);
        }
        ok
    }

    async fn run(&self, ctx: &Ctx, job: Job) {
        Metrics::inc(&ctx.metrics.c.verify_jobs);
        match self.process(ctx, &job.sig).await {
            Ok(Some(outcome)) => {
                ctx.metrics.verify_latency.record(job.enqueued.elapsed());
                tracing::info!(sig = %job.sig, %outcome, "verified");
                self.seen.lock().unwrap().insert_done(job.sig);
            }
            Ok(None) => {
                Metrics::inc(&ctx.metrics.c.verify_rejected);
                self.seen.lock().unwrap().insert_done(job.sig);
            }
            Err(e) if job.attempt + 1 < MAX_ATTEMPTS => {
                Metrics::inc(&ctx.metrics.c.verify_retries);
                tracing::debug!(sig = %job.sig, attempt = job.attempt, "retrying: {e:#}");
                let delay = Duration::from_millis((400u64 << job.attempt).min(8_000));
                let tx = self.tx.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(delay).await;
                    let _ = tx.send(Job { attempt: job.attempt + 1, ..job }).await;
                });
            }
            Err(e) => {
                tracing::warn!(sig = %job.sig, "giving up: {e:#}");
                self.seen.lock().unwrap().inflight.remove(&job.sig);
            }
        }
    }

    /// Ok(Some) applied, Ok(None) not ours / invalid, Err retryable.
    async fn process(&self, ctx: &Ctx, sig: &str) -> Result<Option<String>> {
        let fetched: FetchedTx = ctx.rpc.get_transaction(sig).await?.context("not visible yet")?;
        if fetched.failed {
            return Ok(None);
        }
        let tx = VersionedTransaction::decode(&fetched.bytes).context("undecodable tx")?;
        let keys = resolve_keys(&tx.message, &fetched.loaded_writable, &fetched.loaded_readonly);
        let Some(finding) = inspect(&tx, &keys, &ctx.treasury) else { return Ok(None) };

        match &finding {
            Finding::Fee { blink, lamports_to_treasury } => {
                let Some(row) = db::get_blink(&ctx.db, blink.as_str()).await? else { return Ok(None) };
                if (*lamports_to_treasury as i64) < row.fee_lamports {
                    tracing::warn!(%blink, paid = lamports_to_treasury, want = row.fee_lamports, "fee underpaid");
                    return Ok(None);
                }
                if db::activate(&ctx.db, blink.as_str(), sig).await? {
                    ctx.bus.emit(*blink, EventKind::Activated { fee_lamports: *lamports_to_treasury }).await;
                    Metrics::inc(&ctx.metrics.c.verify_ok);
                    return Ok(Some(format!("fee for {blink}")));
                }
                Metrics::inc(&ctx.metrics.c.verify_dedup);
                Ok(Some(format!("fee for {blink} (already active)")))
            }
            Finding::Payment { blink, payer, .. } => {
                let Some(row) = db::get_blink(&ctx.db, blink.as_str()).await? else { return Ok(None) };
                let Some(amount) = delivered(&finding, &row, &ctx.atas) else { return Ok(None) };
                let amount_i = i64::try_from(amount).unwrap_or(i64::MAX);
                if db::insert_payment(&ctx.db, sig, blink.as_str(), &payer.to_string(), amount_i, fetched.slot as i64).await? {
                    ctx.bus.emit(*blink, EventKind::Paid { amount, payer: payer.0 }).await;
                    Metrics::inc(&ctx.metrics.c.verify_ok);
                    return Ok(Some(format!("payment {amount} to {blink}")));
                }
                Metrics::inc(&ctx.metrics.c.verify_dedup);
                Ok(Some("payment (duplicate)".into()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txbuild;
    use blinker_solana::{programs, Hash, Message, Transaction};

    fn decode(tx: &Transaction) -> VersionedTransaction {
        VersionedTransaction::decode(&tx.serialize()).unwrap()
    }

    #[test]
    fn fee_and_payment_findings() {
        let id = BlinkId::parse("fee000000001").unwrap();
        let (creator, treasury, recipient) = (Pubkey([1; 32]), Pubkey([2; 32]), Pubkey([4; 32]));
        let fee = decode(&txbuild::fee_tx(&creator, &treasury, 1_000_000, &id, Hash::default()));
        let keys = fee.message.inner().account_keys.clone();
        assert_eq!(inspect(&fee, &keys, &treasury), Some(Finding::Fee { blink: id, lamports_to_treasury: 1_000_000 }));

        let pay = decode(&txbuild::sol_payment_tx(&creator, &recipient, 5, &treasury, &id, Hash::default()));
        let keys = pay.message.inner().account_keys.clone();
        match inspect(&pay, &keys, &treasury).unwrap() {
            Finding::Payment { blink, payer, sol, .. } => {
                assert_eq!((blink, payer), (id, creator));
                assert_eq!(sol, vec![(recipient, 5)]);
            }
            f => panic!("{f:?}"),
        }
    }

    #[test]
    fn untagged_tx_is_ignored() {
        let (a, b) = (Pubkey([1; 32]), Pubkey([2; 32]));
        let tx = Transaction::new_unsigned(Message::new(&[programs::system_transfer(&a, &b, 1)], &a, Hash::default()));
        let d = decode(&tx);
        assert_eq!(inspect(&d, &d.message.inner().account_keys.clone(), &b), None);
    }

    #[test]
    fn signature_shape() {
        assert!(is_signature(&bs58::encode([7u8; 64]).into_string()));
        assert!(!is_signature("abc"));
        assert!(!is_signature(&bs58::encode([7u8; 32]).into_string()));
    }
}
