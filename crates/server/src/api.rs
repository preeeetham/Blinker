//! HTTP API: Solana Actions endpoints, blink management, analytics and
//! system introspection.

use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::time::Instant;

use axum::extract::{Path, Query, Request, State};
use axum::http::{header, HeaderName, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use blinker_analytics::{Asset, BlinkId, EventKind};
use blinker_solana::{programs, Pubkey, LAMPORTS_PER_SOL};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::db::{self, BlinkRow, NewBlink};
use crate::metrics::Metrics;
use crate::pipeline::now_ms;
use crate::txbuild::{self, SplPayment};
use crate::AppState;

type S = State<Arc<AppState>>;

// ───────────────────────────── errors ─────────────────────────────

pub struct ApiError(StatusCode, String);

impl ApiError {
    fn bad(msg: impl Into<String>) -> Self {
        Self(StatusCode::BAD_REQUEST, msg.into())
    }
    fn not_found() -> Self {
        Self(StatusCode::NOT_FOUND, "blink not found".into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "message": self.1, "error": { "message": self.1 } }))).into_response()
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        tracing::error!("internal error: {e:#}");
        Self(StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
    }
}

type ApiResult<T> = Result<T, ApiError>;

// ───────────────────────────── router ─────────────────────────────

pub fn router(state: Arc<AppState>) -> Router {
    let actions = Router::new()
        .route("/actions.json", get(actions_json).options(preflight))
        .route("/api/actions/{id}", get(action_get).post(action_post).options(preflight))
        .route("/api/actions/{id}/next", post(action_next).options(preflight))
        .layer(middleware::from_fn_with_state(state.clone(), actions_headers));

    let api = Router::new()
        .route("/api/config", get(config))
        .route("/api/blinks", get(list_blinks).post(create_blink))
        .route("/api/blinks/{id}", get(get_blink))
        .route("/api/blinks/{id}/confirm", post(confirm))
        .route("/api/blinks/{id}/series", get(series))
        .route("/api/tx/send", post(relay_tx))
        .route("/api/stats", get(stats))
        .route("/api/system", get(system))
        .route("/metrics", get(prometheus))
        .route("/healthz", get(|| async { "ok" }));

    actions
        .merge(api)
        .layer(tower_http::limit::RequestBodyLimitLayer::new(64 << 10))
        .layer(middleware::from_fn_with_state(state.clone(), track))
        .with_state(state)
}

/// Request counter + latency histogram for every route.
async fn track(State(st): S, req: Request, next: Next) -> Response {
    let t = Instant::now();
    let res = next.run(req).await;
    Metrics::inc(&st.metrics.c.http_requests);
    st.metrics.http_latency.record(t.elapsed());
    res
}

/// CORS + version headers the Solana Actions spec requires on every
/// response from an action endpoint (errors included).
async fn actions_headers(State(st): S, req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    let h = res.headers_mut();
    let set = |h: &mut axum::http::HeaderMap, k: &'static str, v: &str| {
        h.insert(HeaderName::from_static(k), HeaderValue::from_str(v).unwrap());
    };
    set(h, "access-control-allow-origin", "*");
    set(h, "access-control-allow-methods", "GET,POST,PUT,OPTIONS");
    set(
        h,
        "access-control-allow-headers",
        "Content-Type, Authorization, Content-Encoding, Accept-Encoding, X-Accept-Action-Version, X-Accept-Blockchain-Ids",
    );
    set(h, "access-control-expose-headers", "X-Action-Version, X-Blockchain-Ids");
    set(h, "x-action-version", "2.4");
    set(h, "x-blockchain-ids", st.cfg.cluster.caip2());
    res
}

async fn preflight() -> StatusCode {
    StatusCode::OK
}

// ───────────────────────────── helpers ─────────────────────────────

fn parse_id(id: &str) -> ApiResult<BlinkId> {
    BlinkId::parse(id).ok_or_else(ApiError::not_found)
}

fn parse_pubkey(s: &str, field: &str) -> ApiResult<Pubkey> {
    s.trim().parse().map_err(|_| ApiError::bad(format!("{field} is not a valid Solana address")))
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn base_units_to_ui(amount: u64, decimals: i16) -> f64 {
    amount as f64 / 10f64.powi(decimals as i32)
}

fn fmt_amount(v: f64) -> String {
    let s = format!("{v:.6}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[derive(Serialize)]
pub struct BlinkDto {
    #[serde(flatten)]
    row: BlinkRow,
    action_url: String,
    blink_url: String,
    dial_url: String,
    fee_explorer_url: Option<String>,
}

fn dto(st: &AppState, row: BlinkRow) -> BlinkDto {
    let action_url = format!("{}/api/actions/{}", st.cfg.public_url, row.id);
    let cluster = match st.cfg.cluster {
        crate::config::Cluster::Mainnet => "mainnet",
        _ => "devnet",
    };
    BlinkDto {
        dial_url: format!("https://dial.to/?action=solana-action:{}&cluster={cluster}", percent_encode(&action_url)),
        blink_url: format!("{}/b/{}", st.cfg.public_url, row.id),
        fee_explorer_url: row.fee_signature.as_deref().map(|s| st.cfg.cluster.explorer_tx(s)),
        action_url,
        row,
    }
}

async fn load(st: &AppState, id: &BlinkId) -> ApiResult<BlinkRow> {
    db::get_blink(&st.db, id.as_str()).await?.ok_or_else(ApiError::not_found)
}

// ───────────────────────────── Solana Actions ─────────────────────────────

async fn actions_json() -> Json<Value> {
    Json(json!({
        "rules": [
            { "pathPattern": "/b/*", "apiPath": "/api/actions/*" },
            { "pathPattern": "/api/actions/**", "apiPath": "/api/actions/**" }
        ]
    }))
}

async fn action_get(State(st): S, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    let id = parse_id(&id)?;
    let b = load(&st, &id).await?;
    Metrics::inc(&st.metrics.c.action_gets);
    st.bus.emit(id, EventKind::View).await;

    let base = format!("/api/actions/{id}");
    let unit = b.unit().to_string();
    let mut links: Vec<Value> = b
        .amounts
        .iter()
        .map(|a| json!({ "type": "transaction", "label": format!("{} {unit}", fmt_amount(*a)), "href": format!("{base}?amount={a}") }))
        .collect();
    if b.allow_custom {
        links.push(json!({
            "type": "transaction",
            "label": format!("Send {unit}"),
            "href": format!("{base}?amount={{amount}}"),
            "parameters": [{ "name": "amount", "label": format!("Enter a {unit} amount"), "type": "number", "required": true, "min": 0 }]
        }));
    }
    let mut body = json!({
        "type": "action",
        "icon": b.icon_url,
        "title": b.title,
        "description": b.description,
        "label": b.label,
        "links": { "actions": links },
    });
    if !b.is_active() {
        body["disabled"] = json!(true);
        body["error"] = json!({ "message": "This blink is waiting for its activation fee to confirm." });
    }
    Ok(Json(body))
}

#[derive(Deserialize)]
struct ActionPostBody {
    account: String,
}

#[derive(Deserialize)]
struct AmountQuery {
    amount: Option<String>,
}

async fn action_post(
    State(st): S,
    Path(id): Path<String>,
    Query(q): Query<AmountQuery>,
    Json(body): Json<ActionPostBody>,
) -> ApiResult<Json<Value>> {
    let t = Instant::now();
    let id = parse_id(&id)?;
    let b = load(&st, &id).await?;
    if !b.is_active() {
        return Err(ApiError(StatusCode::FORBIDDEN, "This blink is not active yet.".into()));
    }
    let payer = parse_pubkey(&body.account, "account")?;
    let ui: f64 = q
        .amount
        .as_deref()
        .and_then(|a| a.trim().parse().ok())
        .or_else(|| b.amounts.first().copied())
        .ok_or_else(|| ApiError::bad("amount is required"))?;
    if !b.allow_custom && !b.amounts.iter().any(|a| (a - ui).abs() < 1e-12) {
        return Err(ApiError::bad("custom amounts are disabled for this blink"));
    }
    let amount = programs::ui_to_base_units(ui, b.decimals as u8).ok_or_else(|| ApiError::bad("invalid amount"))?;
    let recipient = parse_pubkey(&b.recipient, "recipient")?;
    if recipient == payer {
        return Err(ApiError::bad("you can't pay your own blink"));
    }
    let bh = st.blockhash.get().await?;

    let tx = if b.is_sol() {
        txbuild::sol_payment_tx(&payer, &recipient, amount, &st.cfg.treasury, &id, bh)
    } else {
        let mint = parse_pubkey(b.mint.as_deref().unwrap_or_default(), "mint")?;
        let tp = parse_pubkey(b.token_program.as_deref().unwrap_or_default(), "token program")?;
        let recipient_ata = st.ata_cache.get(&recipient, &mint, &tp);
        txbuild::spl_payment_tx(
            &SplPayment {
                payer: &payer,
                recipient: &recipient,
                recipient_ata: &recipient_ata,
                mint: &mint,
                token_program: &tp,
                amount,
                decimals: b.decimals as u8,
                reference: &st.cfg.treasury,
            },
            &id,
            bh,
        )
    };

    Metrics::inc(&st.metrics.c.action_posts);
    st.bus.emit(id, EventKind::ActionPost { amount }).await;
    st.metrics.action_post_latency.record(t.elapsed());
    Ok(Json(json!({
        "type": "transaction",
        "transaction": tx.to_base64(),
        "message": format!("Sending {} {} to {}", fmt_amount(ui), b.unit(), b.title),
        "links": { "next": { "type": "post", "href": format!("/api/actions/{id}/next") } }
    })))
}

#[derive(Deserialize)]
struct NextBody {
    #[allow(dead_code)]
    account: Option<String>,
    signature: String,
}

/// Actions-spec callback after the wallet sends the tx: fast-path the
/// verification instead of waiting for the websocket notification.
async fn action_next(State(st): S, Path(id): Path<String>, Json(body): Json<NextBody>) -> ApiResult<Json<Value>> {
    let id = parse_id(&id)?;
    let b = load(&st, &id).await?;
    if !st.verifier.submit(&body.signature) {
        return Err(ApiError::bad("invalid signature"));
    }
    Ok(Json(json!({
        "type": "completed",
        "icon": b.icon_url,
        "title": b.title,
        "description": "Sent! Your payment will show on the blink's live stats once it confirms.",
        "label": "Thank you",
    })))
}

// ───────────────────────────── blink management ─────────────────────────────

#[derive(Deserialize)]
pub struct CreateBlink {
    kind: String,
    title: String,
    description: String,
    #[serde(default)]
    icon_url: String,
    label: String,
    recipient: String,
    creator: String,
    mint: Option<String>,
    token_symbol: Option<String>,
    amounts: Vec<f64>,
    #[serde(default = "yes")]
    allow_custom: bool,
    #[serde(default)]
    theme: Option<String>,
}

fn yes() -> bool {
    true
}

const THEMES: &[&str] = &["aurora", "sunset", "ocean", "lime", "mono"];

fn check_len(v: &str, field: &str, min: usize, max: usize) -> ApiResult<()> {
    let n = v.trim().chars().count();
    if n < min || n > max {
        return Err(ApiError::bad(format!("{field} must be {min}–{max} characters")));
    }
    Ok(())
}

fn valid_icon(url: &str) -> bool {
    url.len() <= 500
        && (url.starts_with("https://") || url.starts_with("http://localhost") || url.starts_with("http://127.0.0.1"))
        && !url.contains(char::is_whitespace)
}

async fn create_blink(State(st): S, Json(req): Json<CreateBlink>) -> ApiResult<Json<Value>> {
    check_len(&req.title, "title", 3, 80)?;
    check_len(&req.description, "description", 3, 400)?;
    check_len(&req.label, "button label", 1, 32)?;
    let recipient = parse_pubkey(&req.recipient, "recipient")?;
    let creator = parse_pubkey(&req.creator, "creator")?;
    if req.amounts.is_empty() || req.amounts.len() > 4 {
        return Err(ApiError::bad("provide 1–4 preset amounts"));
    }
    if req.amounts.iter().any(|a| !a.is_finite() || *a <= 0.0 || *a > 1e12) {
        return Err(ApiError::bad("amounts must be positive numbers"));
    }
    let icon = if req.icon_url.trim().is_empty() {
        format!("{}/blink.svg", st.cfg.public_url)
    } else if valid_icon(req.icon_url.trim()) {
        req.icon_url.trim().to_string()
    } else {
        return Err(ApiError::bad("icon must be an https:// image URL"));
    };
    let theme = req.theme.as_deref().unwrap_or("aurora");
    if !THEMES.contains(&theme) {
        return Err(ApiError::bad("unknown theme"));
    }

    let (kind, asset, mint, token_program, decimals, symbol) = match req.kind.as_str() {
        "sol" => ("sol", Asset::Sol, None, None, 9i16, None),
        "spl" => {
            let mint = parse_pubkey(req.mint.as_deref().unwrap_or_default(), "mint")?;
            let acct = st
                .rpc
                .get_account(&mint)
                .await?
                .ok_or_else(|| ApiError::bad(format!("mint {mint} does not exist on {}", st.cfg.cluster.name())))?;
            if acct.owner != programs::TOKEN_PROGRAM_ID && acct.owner != programs::TOKEN_2022_PROGRAM_ID {
                return Err(ApiError::bad("that address is not an SPL token mint"));
            }
            let d = programs::mint_decimals(&acct.data).ok_or_else(|| ApiError::bad("mint account is not initialized"))?;
            let sym = req.token_symbol.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(|s| s.to_uppercase());
            if sym.as_deref().is_some_and(|s| s.len() > 10 || !s.chars().all(|c| c.is_ascii_alphanumeric())) {
                return Err(ApiError::bad("token symbol must be up to 10 letters/digits"));
            }
            ("spl", Asset::Spl, Some(mint.to_string()), Some(acct.owner.to_string()), d as i16, sym)
        }
        _ => return Err(ApiError::bad("kind must be sol or spl")),
    };
    for a in &req.amounts {
        if programs::ui_to_base_units(*a, decimals as u8).is_none() {
            return Err(ApiError::bad(format!("amount {a} is too small for {decimals} decimals")));
        }
    }

    let id = BlinkId::from_random(rand::random());
    db::insert_blink(
        &st.db,
        &NewBlink {
            id: id.as_str(),
            kind,
            title: req.title.trim(),
            description: req.description.trim(),
            icon_url: &icon,
            label: req.label.trim(),
            recipient: &recipient.to_string(),
            creator: &creator.to_string(),
            mint: mint.as_deref(),
            token_program: token_program.as_deref(),
            token_symbol: symbol.as_deref(),
            decimals,
            amounts: &req.amounts,
            allow_custom: req.allow_custom,
            theme,
            fee_lamports: st.cfg.fee_lamports as i64,
        },
    )
    .await?;
    Metrics::inc(&st.metrics.c.blinks_created);
    st.bus.emit(id, EventKind::Created { asset }).await;

    let bh = st.blockhash.get().await?;
    let tx = txbuild::fee_tx(&creator, &st.cfg.treasury, st.cfg.fee_lamports, &id, bh);
    let row = load(&st, &id).await?;
    Ok(Json(json!({
        "id": id,
        "transaction": tx.to_base64(),
        "fee_lamports": st.cfg.fee_lamports,
        "blink": dto(&st, row),
    })))
}

#[derive(Deserialize)]
struct ConfirmBody {
    signature: String,
}

async fn confirm(State(st): S, Path(id): Path<String>, Json(body): Json<ConfirmBody>) -> ApiResult<(StatusCode, Json<Value>)> {
    let id = parse_id(&id)?;
    load(&st, &id).await?;
    if !st.verifier.submit(body.signature.trim()) {
        return Err(ApiError::bad("invalid or unqueueable signature"));
    }
    Ok((StatusCode::ACCEPTED, Json(json!({ "queued": true }))))
}

async fn get_blink(State(st): S, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    let id = parse_id(&id)?;
    let row = load(&st, &id).await?;
    let payments = db::recent_payments(&st.db, id.as_str(), 10).await?;
    let decimals = row.decimals;
    let payments: Vec<Value> = payments
        .into_iter()
        .map(|p| {
            json!({
                "signature": p.signature,
                "payer": p.payer,
                "amount": fmt_amount(base_units_to_ui(p.amount as u64, decimals)),
                "created_at": p.created_at,
                "explorer_url": st.cfg.cluster.explorer_tx(&p.signature),
            })
        })
        .collect();
    let stats = st.engine.read().unwrap().summary(&id, now_ms());
    Ok(Json(json!({ "blink": dto(&st, row), "stats": stats, "payments": payments })))
}

#[derive(Deserialize)]
struct ListQuery {
    creator: Option<String>,
}

async fn list_blinks(State(st): S, Query(q): Query<ListQuery>) -> ApiResult<Json<Value>> {
    let rows = match q.creator.as_deref() {
        Some(c) => db::list_by_creator(&st.db, &parse_pubkey(c, "creator")?.to_string(), 100).await?,
        None => db::list_recent_active(&st.db, 24).await?,
    };
    let now = now_ms();
    let eng = st.engine.read().unwrap();
    let items: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            let stats = BlinkId::parse(&r.id).and_then(|id| eng.summary(&id, now));
            json!({ "blink": dto(&st, r), "stats": stats })
        })
        .collect();
    Ok(Json(json!({ "items": items })))
}

#[derive(Deserialize)]
struct SeriesQuery {
    res: Option<String>,
}

async fn series(State(st): S, Path(id): Path<String>, Query(q): Query<SeriesQuery>) -> ApiResult<Json<Value>> {
    let id = parse_id(&id)?;
    let hourly = q.res.as_deref() == Some("hour");
    let s = st.engine.read().unwrap().series(&id, now_ms(), hourly).ok_or_else(ApiError::not_found)?;
    Ok(Json(json!({ "resolution": if hourly { "hour" } else { "minute" }, "points": s })))
}

async fn config(State(st): S) -> Json<Value> {
    Json(json!({
        "cluster": st.cfg.cluster.name(),
        "wallet_chain": st.cfg.cluster.wallet_chain(),
        "treasury": st.cfg.treasury,
        "fee_lamports": st.cfg.fee_lamports,
        "fee_sol": st.cfg.fee_lamports as f64 / LAMPORTS_PER_SOL as f64,
        "public_url": st.cfg.public_url,
        "rpc_url": st.cfg.rpc_url,
    }))
}

async fn stats(State(st): S) -> ApiResult<Json<Value>> {
    let now = now_ms();
    let (global, throughput, series, top) = {
        let eng = st.engine.read().unwrap();
        (eng.global(now), eng.throughput(now), eng.global_series(now), eng.top(6, now))
    };
    let mut top_out = Vec::with_capacity(top.len());
    for s in top {
        if let Some(r) = db::get_blink(&st.db, s.id.as_str()).await? {
            top_out.push(json!({ "blink": dto(&st, r), "stats": s }));
        }
    }
    Ok(Json(json!({ "global": global, "throughput": throughput, "series": series, "top": top_out })))
}

async fn system(State(st): S) -> Json<Value> {
    let m = &st.metrics;
    let log = st.log_stats.lock().unwrap().clone();
    let committed = m.committed_offset.load(Relaxed);
    let consumed = m.consumer_offset.load(Relaxed);
    let engine = {
        let e = st.engine.read().unwrap();
        json!({ "events_applied": e.events_applied(), "orphans": e.orphans(), "memory_bytes": e.memory_bytes() })
    };
    Json(json!({
        "cluster": st.cfg.cluster.name(),
        "treasury": st.cfg.treasury,
        "process": st.proc.latest(),
        "pipeline": {
            "queue_depth": m.queue_depth.load(Relaxed),
            "queue_capacity": m.queue_capacity.load(Relaxed),
            "last_batch": m.last_batch.load(Relaxed),
            "max_batch": m.max_batch.load(Relaxed),
            "committed_offset": committed,
            "consumer_offset": consumed,
            "consumer_lag": committed.saturating_sub(consumed),
        },
        "log": {
            "segments": log.segments,
            "bytes": log.bytes,
            "first_offset": log.first_offset,
            "next_offset": log.next_offset,
            "commits": log.commits,
            "fsyncs": log.fsyncs,
            "last_fsync_us": log.last_fsync_micros,
            "recovered_truncated_bytes": log.recovered_truncated_bytes,
        },
        "engine": engine,
        "watcher": { "connected": m.ws_connected.load(Relaxed) == 1 },
        "counters": m.c.snapshot(),
        "latency": {
            "http": m.http_latency.quantiles(),
            "rpc": m.rpc_latency.quantiles(),
            "action_post": m.action_post_latency.quantiles(),
            "log_commit": m.log_commit_latency.quantiles(),
            "verify": m.verify_latency.quantiles(),
        },
        "throughput": st.engine.read().unwrap().throughput(now_ms()),
    }))
}

async fn prometheus(State(st): S) -> impl IntoResponse {
    let p = st.proc.latest();
    let log = st.log_stats.lock().unwrap().clone();
    let body = st.metrics.prometheus(&[
        ("process_cpu_percent", p.cpu_percent),
        ("process_resident_memory_bytes", p.rss_bytes as f64),
        ("process_open_fds", p.open_fds as f64),
        ("blinker_log_bytes", log.bytes as f64),
        ("blinker_log_segments", log.segments as f64),
        ("blinker_log_next_offset", log.next_offset as f64),
    ]);
    ([(header::CONTENT_TYPE, "text/plain; version=0.0.4")], body)
}

#[derive(Deserialize)]
struct RelayBody {
    transaction: String,
}

/// Submits a wallet-signed transaction to the configured cluster. Lets the
/// UI use `signTransaction` (wallet signs only) so the tx lands on *our*
/// cluster regardless of which network the wallet is set to. Only relays
/// Blinker transactions (they all mention the treasury), so it can't be
/// used as a free open RPC.
async fn relay_tx(State(st): S, Json(body): Json<RelayBody>) -> ApiResult<Json<Value>> {
    let tx = blinker_solana::VersionedTransaction::decode_base64(&body.transaction)
        .map_err(|e| ApiError::bad(format!("invalid transaction: {e}")))?;
    if !tx.message.inner().account_keys.contains(&st.cfg.treasury) {
        return Err(ApiError::bad("not a Blinker transaction"));
    }
    if tx.signatures.iter().any(|s| s.iter().all(|b| *b == 0)) {
        return Err(ApiError::bad("transaction is not fully signed"));
    }
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, body.transaction.trim())
        .map_err(|_| ApiError::bad("invalid base64"))?;
    let sig = st
        .rpc
        .send_transaction(&bytes)
        .await
        .map_err(|e| ApiError::bad(format!("cluster rejected the transaction: {e:#}")))?;
    st.verifier.submit(&sig);
    Ok(Json(json!({ "signature": sig })))
}
