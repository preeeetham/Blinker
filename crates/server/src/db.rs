use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::FromRow;

pub async fn connect(url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(16)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct BlinkRow {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub description: String,
    pub icon_url: String,
    pub label: String,
    pub recipient: String,
    pub creator: String,
    pub mint: Option<String>,
    pub token_program: Option<String>,
    pub token_symbol: Option<String>,
    pub decimals: i16,
    pub amounts: Vec<f64>,
    pub allow_custom: bool,
    pub theme: String,
    pub fee_lamports: i64,
    pub status: String,
    pub fee_signature: Option<String>,
    pub created_at: DateTime<Utc>,
    pub activated_at: Option<DateTime<Utc>>,
}

impl BlinkRow {
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }
    pub fn is_sol(&self) -> bool {
        self.kind == "sol"
    }
    /// Display unit, e.g. "SOL" or the token symbol.
    pub fn unit(&self) -> &str {
        if self.is_sol() { "SOL" } else { self.token_symbol.as_deref().unwrap_or("tokens") }
    }
}

pub struct NewBlink<'a> {
    pub id: &'a str,
    pub kind: &'a str,
    pub title: &'a str,
    pub description: &'a str,
    pub icon_url: &'a str,
    pub label: &'a str,
    pub recipient: &'a str,
    pub creator: &'a str,
    pub mint: Option<&'a str>,
    pub token_program: Option<&'a str>,
    pub token_symbol: Option<&'a str>,
    pub decimals: i16,
    pub amounts: &'a [f64],
    pub allow_custom: bool,
    pub theme: &'a str,
    pub fee_lamports: i64,
}

pub async fn insert_blink(db: &PgPool, b: &NewBlink<'_>) -> Result<()> {
    sqlx::query(
        "INSERT INTO blinks (id, kind, title, description, icon_url, label, recipient, creator, mint, token_program,
                             token_symbol, decimals, amounts, allow_custom, theme, fee_lamports)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)",
    )
    .bind(b.id)
    .bind(b.kind)
    .bind(b.title)
    .bind(b.description)
    .bind(b.icon_url)
    .bind(b.label)
    .bind(b.recipient)
    .bind(b.creator)
    .bind(b.mint)
    .bind(b.token_program)
    .bind(b.token_symbol)
    .bind(b.decimals)
    .bind(b.amounts)
    .bind(b.allow_custom)
    .bind(b.theme)
    .bind(b.fee_lamports)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn get_blink(db: &PgPool, id: &str) -> Result<Option<BlinkRow>> {
    Ok(sqlx::query_as::<_, BlinkRow>("SELECT * FROM blinks WHERE id = $1").bind(id).fetch_optional(db).await?)
}

pub async fn list_by_creator(db: &PgPool, creator: &str, limit: i64) -> Result<Vec<BlinkRow>> {
    Ok(sqlx::query_as::<_, BlinkRow>("SELECT * FROM blinks WHERE creator = $1 ORDER BY created_at DESC LIMIT $2")
        .bind(creator)
        .bind(limit)
        .fetch_all(db)
        .await?)
}

pub async fn list_recent_active(db: &PgPool, limit: i64) -> Result<Vec<BlinkRow>> {
    Ok(sqlx::query_as::<_, BlinkRow>(
        "SELECT * FROM blinks WHERE status = 'active' ORDER BY activated_at DESC NULLS LAST LIMIT $1",
    )
    .bind(limit)
    .fetch_all(db)
    .await?)
}

/// Flips a pending blink to active. Returns false if it was already active
/// (idempotent: the watcher and the client callback may both report it).
pub async fn activate(db: &PgPool, id: &str, signature: &str) -> Result<bool> {
    let r = sqlx::query(
        "UPDATE blinks SET status = 'active', fee_signature = $2, activated_at = now()
         WHERE id = $1 AND status = 'pending'",
    )
    .bind(id)
    .bind(signature)
    .execute(db)
    .await?;
    Ok(r.rows_affected() == 1)
}

/// Inserts a payment; false if this signature was already recorded.
pub async fn insert_payment(db: &PgPool, sig: &str, blink: &str, payer: &str, amount: i64, slot: i64) -> Result<bool> {
    let r = sqlx::query(
        "INSERT INTO payments (signature, blink_id, payer, amount, slot) VALUES ($1,$2,$3,$4,$5)
         ON CONFLICT (signature) DO NOTHING",
    )
    .bind(sig)
    .bind(blink)
    .bind(payer)
    .bind(amount)
    .bind(slot)
    .execute(db)
    .await?;
    Ok(r.rows_affected() == 1)
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PaymentRow {
    pub signature: String,
    pub payer: String,
    pub amount: i64,
    pub slot: i64,
    pub created_at: DateTime<Utc>,
}

pub async fn recent_payments(db: &PgPool, blink: &str, limit: i64) -> Result<Vec<PaymentRow>> {
    Ok(sqlx::query_as::<_, PaymentRow>(
        "SELECT signature, payer, amount, slot, created_at FROM payments WHERE blink_id = $1
         ORDER BY created_at DESC LIMIT $2",
    )
    .bind(blink)
    .bind(limit)
    .fetch_all(db)
    .await?)
}

pub async fn all_blinks(db: &PgPool) -> Result<Vec<BlinkRow>> {
    Ok(sqlx::query_as::<_, BlinkRow>("SELECT * FROM blinks ORDER BY created_at").fetch_all(db).await?)
}

/// Every payment for a blink, oldest first.
pub async fn payments_for(db: &PgPool, blink: &str) -> Result<Vec<PaymentRow>> {
    Ok(sqlx::query_as::<_, PaymentRow>(
        "SELECT signature, payer, amount, slot, created_at FROM payments WHERE blink_id = $1 ORDER BY created_at",
    )
    .bind(blink)
    .fetch_all(db)
    .await?)
}
