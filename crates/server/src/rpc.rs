//! Minimal Solana JSON-RPC client with retry/backoff, latency metrics and a
//! background blockhash cache.

use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use base64::Engine as _;
use blinker_solana::{Hash, Pubkey};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::metrics::Metrics;

#[derive(Clone)]
pub struct Rpc {
    http: reqwest::Client,
    url: Arc<str>,
    next_id: Arc<AtomicU64>,
    metrics: Arc<Metrics>,
}

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcErrorBody>,
}

#[derive(Debug, Deserialize)]
struct RpcErrorBody {
    code: i64,
    message: String,
}

/// A fetched transaction, still in wire format.
#[derive(Debug, Clone)]
pub struct FetchedTx {
    pub slot: u64,
    pub bytes: Vec<u8>,
    pub failed: bool,
    pub loaded_writable: Vec<Pubkey>,
    pub loaded_readonly: Vec<Pubkey>,
}

#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub owner: Pubkey,
    pub lamports: u64,
    pub data: Vec<u8>,
}

const MAX_ATTEMPTS: u32 = 4;

impl Rpc {
    pub fn new(url: &str, metrics: Arc<Metrics>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .pool_idle_timeout(Duration::from_secs(60))
            .tcp_nodelay(true)
            .build()
            .expect("reqwest client");
        Self { http, url: url.into(), next_id: Arc::new(AtomicU64::new(1)), metrics }
    }

    /// One JSON-RPC call. Retries transport errors, HTTP 429 and 5xx with
    /// exponential backoff (200ms, 400ms, 800ms) plus jitter; JSON-RPC
    /// errors (bad params etc.) are returned immediately.
    pub async fn call<T: DeserializeOwned>(&self, method: &str, params: Value) -> Result<Option<T>> {
        let body = json!({ "jsonrpc": "2.0", "id": self.next_id.fetch_add(1, Relaxed), "method": method, "params": params });
        let mut attempt = 0;
        loop {
            attempt += 1;
            Metrics::inc(&self.metrics.c.rpc_calls);
            let t = Instant::now();
            let res = self.http.post(&*self.url).json(&body).send().await;
            self.metrics.rpc_latency.record(t.elapsed());
            let retryable = match res {
                Ok(r) if r.status().is_success() => {
                    let parsed: RpcResponse<T> = r.json().await.with_context(|| format!("{method}: bad response"))?;
                    if let Some(e) = parsed.error {
                        Metrics::inc(&self.metrics.c.rpc_errors);
                        bail!("{method}: rpc error {}: {}", e.code, e.message);
                    }
                    return Ok(parsed.result);
                }
                Ok(r) if r.status().as_u16() == 429 || r.status().is_server_error() => anyhow!("{method}: HTTP {}", r.status()),
                Ok(r) => {
                    Metrics::inc(&self.metrics.c.rpc_errors);
                    bail!("{method}: HTTP {}", r.status());
                }
                Err(e) => anyhow!("{method}: {e}"),
            };
            if attempt >= MAX_ATTEMPTS {
                Metrics::inc(&self.metrics.c.rpc_errors);
                return Err(retryable);
            }
            Metrics::inc(&self.metrics.c.rpc_retries);
            let backoff = 100u64 << attempt;
            let jitter = rand::random::<u64>() % (backoff / 2 + 1);
            tokio::time::sleep(Duration::from_millis(backoff + jitter)).await;
        }
    }

    pub async fn latest_blockhash(&self) -> Result<(Hash, u64)> {
        #[derive(Deserialize)]
        struct V {
            blockhash: String,
            #[serde(rename = "lastValidBlockHeight")]
            last_valid: u64,
        }
        #[derive(Deserialize)]
        struct R {
            value: V,
        }
        let r: R = self
            .call("getLatestBlockhash", json!([{ "commitment": "confirmed" }]))
            .await?
            .context("getLatestBlockhash: empty")?;
        Ok((r.value.blockhash.parse().map_err(|e| anyhow!("{e}"))?, r.value.last_valid))
    }

    pub async fn get_transaction(&self, signature: &str) -> Result<Option<FetchedTx>> {
        #[derive(Deserialize)]
        struct Loaded {
            #[serde(default)]
            writable: Vec<Pubkey>,
            #[serde(default)]
            readonly: Vec<Pubkey>,
        }
        #[derive(Deserialize)]
        struct Meta {
            err: Option<Value>,
            #[serde(rename = "loadedAddresses")]
            loaded: Option<Loaded>,
        }
        #[derive(Deserialize)]
        struct R {
            slot: u64,
            transaction: (String, String),
            meta: Option<Meta>,
        }
        let r: Option<R> = self
            .call(
                "getTransaction",
                json!([signature, { "encoding": "base64", "commitment": "confirmed", "maxSupportedTransactionVersion": 0 }]),
            )
            .await?;
        let Some(r) = r else { return Ok(None) };
        let bytes = base64::engine::general_purpose::STANDARD.decode(&r.transaction.0).context("tx base64")?;
        let meta = r.meta;
        let failed = meta.as_ref().map(|m| m.err.is_some()).unwrap_or(true);
        let loaded = meta.and_then(|m| m.loaded);
        let (loaded_writable, loaded_readonly) = loaded.map(|l| (l.writable, l.readonly)).unwrap_or_default();
        Ok(Some(FetchedTx { slot: r.slot, bytes, failed, loaded_writable, loaded_readonly }))
    }

    pub async fn get_account(&self, key: &Pubkey) -> Result<Option<AccountInfo>> {
        #[derive(Deserialize)]
        struct A {
            owner: Pubkey,
            lamports: u64,
            data: (String, String),
        }
        #[derive(Deserialize)]
        struct R {
            value: Option<A>,
        }
        let r: R = self
            .call("getAccountInfo", json!([key.to_string(), { "encoding": "base64", "commitment": "confirmed" }]))
            .await?
            .context("getAccountInfo: empty")?;
        r.value
            .map(|a| {
                Ok(AccountInfo {
                    owner: a.owner,
                    lamports: a.lamports,
                    data: base64::engine::general_purpose::STANDARD.decode(a.data.0)?,
                })
            })
            .transpose()
    }

    pub async fn send_transaction(&self, tx_bytes: &[u8]) -> Result<String> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(tx_bytes);
        self.call::<String>(
            "sendTransaction",
            json!([b64, { "encoding": "base64", "preflightCommitment": "confirmed" }]),
        )
        .await?
        .context("sendTransaction: empty")
    }

    pub async fn balance(&self, key: &Pubkey) -> Result<u64> {
        #[derive(Deserialize)]
        struct R {
            value: u64,
        }
        Ok(self.call::<R>("getBalance", json!([key.to_string(), { "commitment": "confirmed" }])).await?.context("getBalance")?.value)
    }

    pub async fn request_airdrop(&self, key: &Pubkey, lamports: u64) -> Result<String> {
        self.call::<String>("requestAirdrop", json!([key.to_string(), lamports])).await?.context("requestAirdrop: empty")
    }

    pub async fn min_rent(&self, space: usize) -> Result<u64> {
        self.call::<u64>("getMinimumBalanceForRentExemption", json!([space])).await?.context("rent")
    }

    /// Most recent successful signatures touching `key` (newest first).
    pub async fn signatures_for_address(&self, key: &Pubkey, limit: usize) -> Result<Vec<String>> {
        #[derive(Deserialize)]
        struct S {
            signature: String,
            err: Option<Value>,
        }
        let r: Vec<S> = self
            .call("getSignaturesForAddress", json!([key.to_string(), { "limit": limit, "commitment": "confirmed" }]))
            .await?
            .unwrap_or_default();
        Ok(r.into_iter().filter(|s| s.err.is_none()).map(|s| s.signature).collect())
    }

    /// `Some(true)` confirmed ok, `Some(false)` failed, `None` unknown yet.
    pub async fn signature_status(&self, sig: &str) -> Result<Option<bool>> {
        #[derive(Deserialize)]
        struct S {
            err: Option<Value>,
            #[serde(rename = "confirmationStatus")]
            status: Option<String>,
        }
        #[derive(Deserialize)]
        struct R {
            value: Vec<Option<S>>,
        }
        let r: R = self.call("getSignatureStatuses", json!([[sig]])).await?.context("getSignatureStatuses")?;
        Ok(r.value.into_iter().next().flatten().and_then(|s| match (s.err, s.status.as_deref()) {
            (Some(_), _) => Some(false),
            (None, Some("confirmed" | "finalized")) => Some(true),
            _ => None,
        }))
    }
}

/// Keeps a recent blockhash in memory so building an action transaction
/// never waits on RPC. Refreshed every 2s (a blockhash is valid for ~60s).
#[derive(Clone)]
pub struct BlockhashCache {
    inner: Arc<RwLock<Option<(Hash, Instant)>>>,
    rpc: Rpc,
}

impl BlockhashCache {
    pub fn new(rpc: Rpc) -> Self {
        Self { inner: Arc::new(RwLock::new(None)), rpc }
    }

    pub fn spawn_refresher(&self, metrics: Arc<Metrics>) {
        let me = self.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(2));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                tick.tick().await;
                match me.rpc.latest_blockhash().await {
                    Ok((h, _)) => {
                        *me.inner.write().unwrap() = Some((h, Instant::now()));
                        Metrics::inc(&metrics.c.blockhash_refreshes);
                    }
                    Err(e) => tracing::warn!("blockhash refresh failed: {e:#}"),
                }
            }
        });
    }

    /// Cached hash if younger than 20s, else a synchronous fetch.
    pub async fn get(&self) -> Result<Hash> {
        if let Some((h, at)) = *self.inner.read().unwrap()
            && at.elapsed() < Duration::from_secs(20)
        {
            return Ok(h);
        }
        let (h, _) = self.rpc.latest_blockhash().await?;
        *self.inner.write().unwrap() = Some((h, Instant::now()));
        Ok(h)
    }
}
