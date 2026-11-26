//! `blinker` — developer CLI. Acts as a headless wallet so the whole system
//! (fee payment → watcher → activation → payments → analytics) can be
//! exercised without a browser extension.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use blinker_server::metrics::Metrics;
use blinker_server::rpc::Rpc;
use blinker_solana::transaction::VersionedTransaction;
use blinker_solana::{programs, AccountMeta, Instruction, Keypair, Message, Pubkey, Transaction, VersionedMessage, LAMPORTS_PER_SOL};
use clap::{Parser, Subcommand};
use serde_json::{json, Value};

#[derive(Parser)]
#[command(name = "blinker", about = "Blinker developer tooling")]
struct Cli {
    /// Blinker server base URL.
    #[arg(long, env = "PUBLIC_URL", default_value = "http://localhost:3000", global = true)]
    server: String,
    /// Solana RPC URL.
    #[arg(long, env = "SOLANA_RPC_URL", default_value = "http://127.0.0.1:8899", global = true)]
    rpc: String,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Create wallets in .keys/ (if missing) and write .env with their secrets.
    Setup {
        #[arg(long, default_value = "localnet")]
        cluster: String,
        #[arg(long, default_value = "postgres://localhost:5432/blinker")]
        database_url: String,
    },
    /// Show SOL balances of the .keys wallets.
    Balances,
    /// Request an airdrop (localnet/devnet faucets only).
    Airdrop {
        #[arg(long, default_value = ".keys/tester.json")]
        keypair: PathBuf,
        #[arg(long, default_value_t = 2.0)]
        sol: f64,
    },
    /// Create a test SPL mint and mint tokens to the payer.
    MintToken {
        #[arg(long, default_value = ".keys/tester.json")]
        payer: PathBuf,
        #[arg(long, default_value_t = 6)]
        decimals: u8,
        #[arg(long, default_value_t = 1_000_000.0)]
        amount: f64,
    },
    /// Create a blink and pay its activation fee.
    CreateBlink {
        #[arg(long, default_value = ".keys/creator.json")]
        keypair: PathBuf,
        #[arg(long, default_value = "sol")]
        kind: String,
        #[arg(long)]
        mint: Option<String>,
        #[arg(long, default_value = "Buy me a coffee")]
        title: String,
        #[arg(long, default_value = "0.01,0.05,0.1")]
        amounts: String,
    },
    /// Pay a blink through its Solana Action, like a wallet would.
    Pay {
        #[arg(long, default_value = ".keys/tester.json")]
        keypair: PathBuf,
        #[arg(long)]
        blink: String,
        #[arg(long)]
        amount: f64,
    },
    /// Full scripted run: SOL + SPL blinks, fees, payments, analytics checks.
    E2e,
}

fn load_keypair(p: &Path) -> Result<Keypair> {
    let s = std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?;
    Keypair::from_json(&s).map_err(|e| anyhow::anyhow!("{}: {e}", p.display()))
}

struct Ctx {
    http: reqwest::Client,
    server: String,
    rpc: Rpc,
}

impl Ctx {
    async fn get(&self, path: &str) -> Result<Value> {
        let r = self.http.get(format!("{}{path}", self.server)).send().await?;
        let status = r.status();
        let v: Value = r.json().await?;
        if !status.is_success() {
            bail!("GET {path}: {status} {v}");
        }
        Ok(v)
    }

    async fn post(&self, path: &str, body: Value) -> Result<Value> {
        let r = self.http.post(format!("{}{path}", self.server)).json(&body).send().await?;
        let status = r.status();
        let v: Value = r.json().await?;
        if !status.is_success() {
            bail!("POST {path}: {status} {v}");
        }
        Ok(v)
    }

    /// Decode a server-built tx, check the wallet is the fee payer, sign, send.
    async fn sign_and_send(&self, b64: &str, signers: &[&Keypair]) -> Result<String> {
        let vt = VersionedTransaction::decode_base64(b64).context("server returned an undecodable tx")?;
        let VersionedMessage::Legacy(message) = vt.message else { bail!("expected legacy tx") };
        if message.fee_payer() != Some(&signers[0].pubkey()) {
            bail!("fee payer mismatch");
        }
        let mut tx = Transaction { signatures: vt.signatures, message };
        tx.sign(signers);
        let sig = self.rpc.send_transaction(&tx.serialize()).await?;
        self.wait_confirmed(&sig).await?;
        Ok(sig)
    }

    /// Same as a browser wallet using `signTransaction`: sign locally, then
    /// submit through Blinker's relay endpoint instead of RPC directly.
    async fn sign_and_relay(&self, b64: &str, signer: &Keypair) -> Result<String> {
        let vt = VersionedTransaction::decode_base64(b64)?;
        let VersionedMessage::Legacy(message) = vt.message else { bail!("expected legacy tx") };
        let mut tx = Transaction { signatures: vt.signatures, message };
        tx.sign(&[signer]);
        let res = self.post("/api/tx/send", json!({ "transaction": tx.to_base64() })).await?;
        let sig = res["signature"].as_str().context("relay: no signature")?.to_string();
        self.wait_confirmed(&sig).await?;
        Ok(sig)
    }

    async fn wait_confirmed(&self, sig: &str) -> Result<()> {
        let t = Instant::now();
        while t.elapsed() < Duration::from_secs(60) {
            match self.rpc.signature_status(sig).await? {
                Some(true) => return Ok(()),
                Some(false) => bail!("transaction {sig} failed on-chain"),
                None => tokio::time::sleep(Duration::from_millis(400)).await,
            }
        }
        bail!("timed out waiting for {sig}")
    }

    async fn send_ixs(&self, ixs: &[Instruction], signers: &[&Keypair]) -> Result<String> {
        let (bh, _) = self.rpc.latest_blockhash().await?;
        let mut tx = Transaction::new_unsigned(Message::new(ixs, &signers[0].pubkey(), bh));
        tx.sign(signers);
        let sig = self.rpc.send_transaction(&tx.serialize()).await?;
        self.wait_confirmed(&sig).await?;
        Ok(sig)
    }

    async fn create_blink(&self, creator: &Keypair, kind: &str, mint: Option<&str>, title: &str, amounts: &[f64]) -> Result<Value> {
        let res = self
            .post(
                "/api/blinks",
                json!({
                    "kind": kind, "title": title,
                    "description": format!("{title} — created by the blinker CLI"),
                    "label": "Support", "recipient": creator.pubkey().to_string(),
                    "creator": creator.pubkey().to_string(), "mint": mint,
                    "token_symbol": mint.map(|_| "TEST"), "amounts": amounts, "theme": "aurora",
                }),
            )
            .await?;
        let id = res["id"].as_str().context("id")?.to_string();
        let sig = self.sign_and_send(res["transaction"].as_str().context("tx")?, &[creator]).await?;
        println!("  fee paid  {sig}");
        self.post(&format!("/api/blinks/{id}/confirm"), json!({ "signature": sig })).await?;
        let t = Instant::now();
        loop {
            let b = self.get(&format!("/api/blinks/{id}")).await?;
            if b["blink"]["status"] == "active" {
                println!("  activated in {} ms (verifier)", t.elapsed().as_millis());
                return Ok(b);
            }
            if t.elapsed() > Duration::from_secs(45) {
                bail!("blink {id} never activated");
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    }

    async fn pay(&self, payer: &Keypair, id: &str, amount: f64) -> Result<String> {
        self.pay_with(payer, id, amount, false).await
    }

    async fn pay_with(&self, payer: &Keypair, id: &str, amount: f64, relay: bool) -> Result<String> {
        let action = self.get(&format!("/api/actions/{id}")).await?;
        if action["disabled"] == true {
            bail!("blink is disabled: {}", action["error"]["message"]);
        }
        let res = self.post(&format!("/api/actions/{id}?amount={amount}"), json!({ "account": payer.pubkey().to_string() })).await?;
        let tx = res["transaction"].as_str().context("tx")?;
        let sig = if relay { self.sign_and_relay(tx, payer).await? } else { self.sign_and_send(tx, &[payer]).await? };
        if let Some(next) = res["links"]["next"]["href"].as_str() {
            self.post(next, json!({ "account": payer.pubkey().to_string(), "signature": sig })).await?;
        }
        Ok(sig)
    }

    async fn mint_token(&self, payer: &Keypair, decimals: u8, ui_amount: f64) -> Result<Pubkey> {
        let mint = Keypair::generate();
        let tp = programs::TOKEN_PROGRAM_ID;
        let rent = self.rpc.min_rent(82).await?;
        let mut create = Vec::with_capacity(52);
        create.extend_from_slice(&0u32.to_le_bytes());
        create.extend_from_slice(&rent.to_le_bytes());
        create.extend_from_slice(&82u64.to_le_bytes());
        create.extend_from_slice(&tp.0);
        let mut init = vec![20, decimals];
        init.extend_from_slice(&payer.pubkey().0);
        init.push(0); // no freeze authority
        let ata = programs::associated_token_address(&payer.pubkey(), &mint.pubkey(), &tp);
        let mut mint_to = vec![7];
        mint_to.extend_from_slice(&programs::ui_to_base_units(ui_amount, decimals).context("amount")?.to_le_bytes());
        let ixs = [
            Instruction {
                program_id: programs::SYSTEM_PROGRAM_ID,
                accounts: vec![AccountMeta::writable(payer.pubkey(), true), AccountMeta::writable(mint.pubkey(), true)],
                data: create,
            },
            Instruction { program_id: tp, accounts: vec![AccountMeta::writable(mint.pubkey(), false)], data: init },
            programs::create_ata_idempotent(&payer.pubkey(), &payer.pubkey(), &mint.pubkey(), &tp),
            Instruction {
                program_id: tp,
                accounts: vec![
                    AccountMeta::writable(mint.pubkey(), false),
                    AccountMeta::writable(ata, false),
                    AccountMeta::readonly(payer.pubkey(), true),
                ],
                data: mint_to,
            },
        ];
        self.send_ixs(&ixs, &[payer, &mint]).await?;
        Ok(mint.pubkey())
    }

    async fn ensure_funded(&self, kp: &Keypair, min_sol: f64) -> Result<()> {
        let bal = self.rpc.balance(&kp.pubkey()).await?;
        if (bal as f64) < min_sol * LAMPORTS_PER_SOL as f64 {
            println!("  airdropping 5 SOL to {}", kp.pubkey());
            let sig = self.rpc.request_airdrop(&kp.pubkey(), 5 * LAMPORTS_PER_SOL).await?;
            self.wait_confirmed(&sig).await?;
        }
        Ok(())
    }
}

fn setup(cluster: &str, database_url: &str) -> Result<()> {
    std::fs::create_dir_all(".keys")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(".keys", std::fs::Permissions::from_mode(0o700))?;
    }
    let mut lines = vec![
        "# Blinker local config + wallet secrets. GITIGNORED — never commit this file.".to_string(),
        "# Keypair JSON files (solana-keygen format) live in .keys/.".into(),
        format!("DATABASE_URL={database_url}"),
        format!("SOLANA_CLUSTER={cluster}"),
    ];
    if cluster != "localnet" {
        lines.push(format!("SOLANA_RPC_URL={}", blinker_server::Cluster::parse(cluster)?.default_rpc()));
    } else {
        lines.push("SOLANA_RPC_URL=http://127.0.0.1:8899".into());
    }
    lines.push("PUBLIC_URL=http://localhost:3000".into());
    lines.push("RUST_LOG=info,blinker_server=debug".into());
    for name in ["treasury", "creator", "tester"] {
        let path = PathBuf::from(format!(".keys/{name}.json"));
        let kp = if path.exists() {
            load_keypair(&path)?
        } else {
            let kp = Keypair::generate();
            std::fs::write(&path, kp.to_json())?;
            kp
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }
        let up = name.to_uppercase();
        lines.push(format!("{up}_PUBKEY={}", kp.pubkey()));
        lines.push(format!("{up}_SECRET_KEY={}", kp.to_base58()));
        println!("{name:>9}  {}", kp.pubkey());
    }
    if Path::new(".env").exists() {
        std::fs::copy(".env", ".env.bak")?;
    }
    std::fs::write(".env", lines.join("\n") + "\n")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(".env", std::fs::Permissions::from_mode(0o600))?;
    }
    println!("wrote .env (mode 600)");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();
    let ctx = Ctx {
        http: reqwest::Client::new(),
        server: cli.server.trim_end_matches('/').to_string(),
        rpc: Rpc::new(&cli.rpc, Arc::new(Metrics::default())),
    };
    match cli.cmd {
        Cmd::Setup { cluster, database_url } => setup(&cluster, &database_url)?,
        Cmd::Balances => {
            for name in ["treasury", "creator", "tester"] {
                let kp = load_keypair(Path::new(&format!(".keys/{name}.json")))?;
                let bal = ctx.rpc.balance(&kp.pubkey()).await?;
                println!("{name:>9}  {}  {:.4} SOL", kp.pubkey(), bal as f64 / LAMPORTS_PER_SOL as f64);
            }
        }
        Cmd::Airdrop { keypair, sol } => {
            let kp = load_keypair(&keypair)?;
            let sig = ctx.rpc.request_airdrop(&kp.pubkey(), (sol * LAMPORTS_PER_SOL as f64) as u64).await?;
            ctx.wait_confirmed(&sig).await?;
            println!("airdropped {sol} SOL to {}: {sig}", kp.pubkey());
        }
        Cmd::MintToken { payer, decimals, amount } => {
            let kp = load_keypair(&payer)?;
            println!("mint {}", ctx.mint_token(&kp, decimals, amount).await?);
        }
        Cmd::CreateBlink { keypair, kind, mint, title, amounts } => {
            let kp = load_keypair(&keypair)?;
            let amounts: Vec<f64> = amounts.split(',').map(|a| a.trim().parse()).collect::<Result<_, _>>()?;
            let b = ctx.create_blink(&kp, &kind, mint.as_deref(), &title, &amounts).await?;
            println!("{}", serde_json::to_string_pretty(&b["blink"])?);
        }
        Cmd::Pay { keypair, blink, amount } => {
            let kp = load_keypair(&keypair)?;
            println!("paid: {}", ctx.pay(&kp, &blink, amount).await?);
        }
        Cmd::E2e => e2e(&ctx).await?,
    }
    Ok(())
}

async fn wait_stat(ctx: &Ctx, id: &str, paid: u64) -> Result<Value> {
    let t = Instant::now();
    loop {
        let b = ctx.get(&format!("/api/blinks/{id}")).await?;
        if b["stats"]["paid"].as_u64().unwrap_or(0) >= paid {
            return Ok(b);
        }
        if t.elapsed() > Duration::from_secs(45) {
            bail!("payments for {id} never showed up in analytics: {}", b["stats"]);
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}

async fn e2e(ctx: &Ctx) -> Result<()> {
    let creator = load_keypair(Path::new(".keys/creator.json"))?;
    let tester = load_keypair(Path::new(".keys/tester.json"))?;
    ctx.get("/healthz").await.ok();
    println!("① funding wallets");
    ctx.ensure_funded(&creator, 1.0).await?;
    ctx.ensure_funded(&tester, 1.0).await?;

    println!("② SOL blink: create + activation fee");
    let b = ctx.create_blink(&creator, "sol", None, "E2E coffee fund", &[0.01, 0.05]).await?;
    let id = b["blink"]["id"].as_str().unwrap().to_string();

    println!("③ paying it twice through the Solana Action (direct RPC, then the UI's relay path)");
    let before = ctx.rpc.balance(&creator.pubkey()).await?;
    ctx.pay(&tester, &id, 0.01).await?;
    ctx.pay_with(&tester, &id, 0.05, true).await?;
    // The relay must refuse non-Blinker transactions.
    let (bh, _) = ctx.rpc.latest_blockhash().await?;
    let mut foreign = Transaction::new_unsigned(Message::new(&[programs::system_transfer(&tester.pubkey(), &creator.pubkey(), 1)], &tester.pubkey(), bh));
    foreign.sign(&[&tester]);
    if ctx.post("/api/tx/send", json!({ "transaction": foreign.to_base64() })).await.is_ok() {
        bail!("relay accepted a non-Blinker transaction");
    }
    let s = wait_stat(ctx, &id, 2).await?;
    let after = ctx.rpc.balance(&creator.pubkey()).await?;
    if after - before != 60_000_000 {
        bail!("recipient balance moved by {} lamports, want 60000000", after - before);
    }
    println!("  stats: views={} posts={} paid={} volume={} lamports", s["stats"]["views"], s["stats"]["posts"], s["stats"]["paid"], s["stats"]["volume"]);

    println!("④ SPL blink: mint a test token, create, pay");
    let mint = ctx.mint_token(&tester, 6, 1_000.0).await?;
    println!("  mint {mint}");
    let b = ctx.create_blink(&creator, "spl", Some(&mint.to_string()), "E2E token tip jar", &[1.5, 10.0]).await?;
    let spl_id = b["blink"]["id"].as_str().unwrap().to_string();
    ctx.pay(&tester, &spl_id, 1.5).await?;
    let s = wait_stat(ctx, &spl_id, 1).await?;
    if s["stats"]["volume"] != "1500000" {
        bail!("SPL volume {} != 1500000", s["stats"]["volume"]);
    }
    println!("  stats: paid={} volume={} base units", s["stats"]["paid"], s["stats"]["volume"]);

    println!("⑤ negative checks");
    let r = ctx.post(&format!("/api/actions/{id}?amount=-1"), json!({ "account": tester.pubkey().to_string() })).await;
    if r.is_ok() {
        bail!("negative amount accepted");
    }
    let r = ctx.post(&format!("/api/actions/{id}"), json!({ "account": "not-a-key" })).await;
    if r.is_ok() {
        bail!("bad pubkey accepted");
    }

    let sys = ctx.get("/api/system").await?;
    println!(
        "⑥ system: events committed={} log fsyncs={} verify ok={} dedup={} ws connected={}",
        sys["counters"]["events_committed"],
        sys["log"]["fsyncs"],
        sys["counters"]["verify_ok"],
        sys["counters"]["verify_dedup"],
        sys["watcher"]["connected"]
    );
    println!("\n✅ e2e passed — SOL blink {id}, SPL blink {spl_id}");
    Ok(())
}
