//! Typed client for Blinker's JSON API (browser side).

use gloo_net::http::{Request, Response};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct AppConfig {
    pub cluster: String,
    pub wallet_chain: String,
    pub treasury: String,
    pub fee_lamports: u64,
    pub fee_sol: f64,
    pub public_url: String,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct Blink {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub description: String,
    pub icon_url: String,
    pub label: String,
    pub recipient: String,
    pub creator: String,
    pub mint: Option<String>,
    pub token_symbol: Option<String>,
    pub decimals: i16,
    pub amounts: Vec<f64>,
    pub allow_custom: bool,
    pub theme: String,
    pub status: String,
    pub fee_lamports: i64,
    pub fee_signature: Option<String>,
    pub created_at: String,
    pub activated_at: Option<String>,
    pub action_url: String,
    pub blink_url: String,
    pub dial_url: String,
    pub fee_explorer_url: Option<String>,
}

impl Blink {
    pub fn unit(&self) -> String {
        if self.kind == "sol" { "SOL".into() } else { self.token_symbol.clone().unwrap_or_else(|| "tokens".into()) }
    }
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }
    /// Volume (base units string) → display amount.
    pub fn ui_volume(&self, base: &str) -> f64 {
        base.parse::<f64>().unwrap_or(0.0) / 10f64.powi(self.decimals as i32)
    }
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct Stats {
    pub views: u64,
    pub posts: u64,
    pub paid: u64,
    pub volume: String,
    pub unique_payers: u64,
    pub conversion: f64,
    pub views_24h: u64,
    pub paid_24h: u64,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct BlinkWithStats {
    pub blink: Blink,
    pub stats: Option<Stats>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct Payment {
    pub signature: String,
    pub payer: String,
    pub amount: String,
    pub created_at: String,
    pub explorer_url: String,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct BlinkDetail {
    pub blink: Blink,
    pub stats: Option<Stats>,
    pub payments: Vec<Payment>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct Global {
    pub blinks: u64,
    pub active_blinks: u64,
    pub views: u64,
    pub posts: u64,
    pub paid: u64,
    pub sol_volume_lamports: String,
    pub unique_payers: u64,
    pub events_applied: u64,
    pub events_per_sec_1m: f64,
    pub engine_memory_bytes: u64,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct SeriesPoint {
    pub t: u64,
    pub views: u32,
    pub posts: u32,
    pub paid: u32,
    pub volume: u64,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct StatsResponse {
    pub global: Global,
    pub throughput: Vec<(u64, u32)>,
    pub series: Vec<SeriesPoint>,
    pub top: Vec<BlinkWithStats>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct ListResponse {
    pub items: Vec<BlinkWithStats>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct SeriesResponse {
    pub points: Vec<SeriesPoint>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CreateBlink {
    pub kind: String,
    pub title: String,
    pub description: String,
    pub icon_url: String,
    pub label: String,
    pub recipient: String,
    pub creator: String,
    pub mint: Option<String>,
    pub token_symbol: Option<String>,
    pub amounts: Vec<f64>,
    pub allow_custom: bool,
    pub theme: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct CreateResponse {
    pub id: String,
    pub transaction: String,
    pub fee_lamports: u64,
    pub blink: Blink,
}

async fn error_of(r: Response) -> String {
    let status = r.status();
    match r.json::<Value>().await {
        Ok(v) => v["message"].as_str().map(String::from).unwrap_or_else(|| format!("HTTP {status}")),
        Err(_) => format!("HTTP {status}"),
    }
}

pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let r = Request::get(path).send().await.map_err(|e| e.to_string())?;
    if !r.ok() {
        return Err(error_of(r).await);
    }
    r.json::<T>().await.map_err(|e| e.to_string())
}

pub async fn post<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, String> {
    let r = Request::post(path).json(body).map_err(|e| e.to_string())?.send().await.map_err(|e| e.to_string())?;
    if !r.ok() {
        return Err(error_of(r).await);
    }
    r.json::<T>().await.map_err(|e| e.to_string())
}
