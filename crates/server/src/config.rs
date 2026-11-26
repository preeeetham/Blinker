use std::path::PathBuf;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use blinker_eventlog::SyncPolicy;
use blinker_solana::Pubkey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cluster {
    Localnet,
    Devnet,
    Mainnet,
}

impl Cluster {
    pub fn parse(s: &str) -> Result<Self> {
        Ok(match s.trim().to_ascii_lowercase().as_str() {
            "localnet" | "localhost" | "local" => Cluster::Localnet,
            "devnet" => Cluster::Devnet,
            "mainnet" | "mainnet-beta" => Cluster::Mainnet,
            other => bail!("unknown SOLANA_CLUSTER {other:?}"),
        })
    }

    pub fn default_rpc(self) -> &'static str {
        match self {
            Cluster::Localnet => "http://127.0.0.1:8899",
            Cluster::Devnet => "https://api.devnet.solana.com",
            Cluster::Mainnet => "https://api.mainnet-beta.solana.com",
        }
    }

    /// Wallet Standard chain id (what the browser wallet is asked to use).
    pub fn wallet_chain(self) -> &'static str {
        match self {
            Cluster::Localnet => "solana:localnet",
            Cluster::Devnet => "solana:devnet",
            Cluster::Mainnet => "solana:mainnet",
        }
    }

    /// CAIP-2 id for the Actions `X-Blockchain-Ids` header.
    pub fn caip2(self) -> &'static str {
        match self {
            Cluster::Localnet | Cluster::Devnet => "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1",
            Cluster::Mainnet => "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Cluster::Localnet => "localnet",
            Cluster::Devnet => "devnet",
            Cluster::Mainnet => "mainnet",
        }
    }

    /// Explorer link for a transaction.
    pub fn explorer_tx(self, sig: &str) -> String {
        match self {
            Cluster::Mainnet => format!("https://explorer.solana.com/tx/{sig}"),
            Cluster::Devnet => format!("https://explorer.solana.com/tx/{sig}?cluster=devnet"),
            Cluster::Localnet => format!(
                "https://explorer.solana.com/tx/{sig}?cluster=custom&customUrl=http%3A%2F%2F127.0.0.1%3A8899"
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub cluster: Cluster,
    pub rpc_url: String,
    pub ws_url: String,
    pub treasury: Pubkey,
    /// Base URL blinks are shared under, e.g. https://blinker.example.
    pub public_url: String,
    pub fee_lamports: u64,
    pub data_dir: PathBuf,
    pub event_queue_capacity: usize,
    pub log_sync: SyncPolicy,
    pub log_segment_bytes: u64,
    pub verify_workers: usize,
}

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> Result<T>
where
    T::Err: std::fmt::Display,
{
    match env(key) {
        Some(v) => v.trim().parse().map_err(|e| anyhow::anyhow!("{key}: {e}")),
        None => Ok(default),
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let cluster = Cluster::parse(&env("SOLANA_CLUSTER").unwrap_or_else(|| "devnet".into()))?;
        let rpc_url = env("SOLANA_RPC_URL").unwrap_or_else(|| cluster.default_rpc().into());
        let ws_url = env("SOLANA_WS_URL").unwrap_or_else(|| derive_ws(&rpc_url));
        let treasury: Pubkey = env("TREASURY_PUBKEY")
            .context("TREASURY_PUBKEY is required (run `cargo run -p blinker-cli -- setup`)")?
            .parse()
            .map_err(|e| anyhow::anyhow!("TREASURY_PUBKEY: {e}"))?;
        let log_sync = match env("EVENT_LOG_SYNC").as_deref().unwrap_or("every") {
            "every" => SyncPolicy::EveryCommit,
            "never" => SyncPolicy::Never,
            ms => SyncPolicy::Interval(Duration::from_millis(
                ms.parse().context("EVENT_LOG_SYNC must be every|never|<millis>")?,
            )),
        };
        Ok(Self {
            database_url: env("DATABASE_URL").context("DATABASE_URL is required")?,
            cluster,
            rpc_url,
            ws_url,
            treasury,
            // Render (and similar hosts) inject the public URL of the service.
            public_url: env("PUBLIC_URL")
                .or_else(|| env("RENDER_EXTERNAL_URL"))
                .unwrap_or_else(|| "http://localhost:3000".into())
                .trim_end_matches('/')
                .into(),
            fee_lamports: env_parse("BLINK_FEE_LAMPORTS", 1_000_000)?,
            data_dir: env("DATA_DIR").unwrap_or_else(|| "data".into()).into(),
            event_queue_capacity: env_parse("EVENT_QUEUE_CAPACITY", 8192)?,
            log_sync,
            log_segment_bytes: env_parse("EVENT_LOG_SEGMENT_BYTES", 16 << 20)?,
            verify_workers: env_parse("VERIFY_WORKERS", 4)?,
        })
    }
}

/// http(s)://host:8899 → ws(s)://host:8900 (Solana's RPC/pubsub port pair).
fn derive_ws(rpc: &str) -> String {
    let ws = rpc.replacen("https://", "wss://", 1).replacen("http://", "ws://", 1);
    ws.replace(":8899", ":8900")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_derivation() {
        assert_eq!(derive_ws("http://127.0.0.1:8899"), "ws://127.0.0.1:8900");
        assert_eq!(derive_ws("https://api.devnet.solana.com"), "wss://api.devnet.solana.com");
    }
}
