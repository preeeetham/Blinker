//! Wallet state + the Wallet Standard bridge (see `js/wallet.js`).

use base64::Engine as _;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Deserialize;
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

use crate::api::{self, AppConfig};
use crate::util::{short, toast};

#[wasm_bindgen(module = "/js/wallet.js")]
extern "C" {
    fn list_wallets() -> String;
    fn remembered() -> String;
    #[wasm_bindgen(catch)]
    async fn connect(name: &str, silent: bool) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    async fn disconnect() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    async fn sign(tx_b64: &str, chain: &str) -> Result<JsValue, JsValue>;
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct WalletInfo {
    pub name: String,
    pub icon: String,
}

#[derive(Clone, Copy)]
pub struct Wallet {
    pub address: RwSignal<Option<String>>,
    pub modal: RwSignal<bool>,
    pub available: RwSignal<Vec<WalletInfo>>,
    pub connecting: RwSignal<bool>,
    pub config: RwSignal<AppConfig>,
}

pub fn use_wallet() -> Wallet {
    expect_context::<Wallet>()
}

fn js_err(e: JsValue) -> String {
    js_sys::Reflect::get(&e, &"message".into())
        .ok()
        .and_then(|m| m.as_string())
        .or_else(|| e.as_string())
        .unwrap_or_else(|| "wallet error".into())
}

impl Wallet {
    pub fn provide() -> Self {
        let w = Wallet {
            address: RwSignal::new(None),
            modal: RwSignal::new(false),
            available: RwSignal::new(vec![]),
            connecting: RwSignal::new(false),
            config: RwSignal::new(AppConfig::default()),
        };
        provide_context(w);
        // Client only: load config, discover wallets, silently reconnect.
        Effect::new(move |_| {
            spawn_local(async move {
                if let Ok(c) = api::get::<AppConfig>("/api/config").await {
                    w.config.set(c);
                }
                // Extensions register asynchronously after app-ready.
                gloo_timer(250).await;
                w.refresh();
                let name = remembered();
                if !name.is_empty()
                    && let Ok(addr) = connect(&name, true).await
                {
                    w.address.set(addr.as_string());
                }
            });
        });
        w
    }

    pub fn refresh(&self) {
        let list: Vec<WalletInfo> = serde_json::from_str(&list_wallets()).unwrap_or_default();
        self.available.set(list);
    }

    pub fn open(&self) {
        self.refresh();
        self.modal.set(true);
    }

    pub fn connect(&self, name: String) {
        let w = *self;
        w.connecting.set(true);
        spawn_local(async move {
            match connect(&name, false).await {
                Ok(a) => {
                    let a = a.as_string().unwrap_or_default();
                    toast(true, format!("Connected {}", short(&a)));
                    w.address.set(Some(a));
                    w.modal.set(false);
                }
                Err(e) => toast(false, js_err(e)),
            }
            w.connecting.set(false);
        });
    }

    pub fn disconnect(&self) {
        let w = *self;
        spawn_local(async move {
            let _ = disconnect().await;
            w.address.set(None);
        });
    }

    /// Has the wallet sign `tx_b64`, submits it to Blinker's cluster, and
    /// returns the base58 signature.
    pub async fn sign_and_submit(&self, tx_b64: &str) -> Result<String, String> {
        let chain = self.config.get_untracked().wallet_chain;
        let chain = if chain.is_empty() { "solana:devnet".to_string() } else { chain };
        let out = sign(tx_b64, &chain).await.map_err(js_err)?;
        let v: Value = serde_json::from_str(&out.as_string().unwrap_or_default()).map_err(|e| e.to_string())?;
        match v["mode"].as_str() {
            Some("signed") => {
                let res: Value = api::post("/api/tx/send", &json!({ "transaction": v["tx"] })).await?;
                res["signature"].as_str().map(String::from).ok_or_else(|| "relay returned no signature".into())
            }
            Some("sent") => {
                let raw = base64::engine::general_purpose::STANDARD
                    .decode(v["signature"].as_str().unwrap_or_default())
                    .map_err(|e| e.to_string())?;
                Ok(bs58::encode(raw).into_string())
            }
            _ => Err("unexpected wallet response".into()),
        }
    }
}

pub async fn gloo_timer(ms: u32) {
    let p = js_sys::Promise::new(&mut |resolve, _| {
        if let Some(w) = web_sys::window() {
            let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms as i32);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(p).await;
}

#[component]
pub fn WalletButton() -> impl IntoView {
    let w = use_wallet();
    view! {
        {move || match w.address.get() {
            Some(a) => {
                view! {
                    <div class="wallet-pill">
                        <span class="dot live"></span>
                        <span class="mono">{short(&a)}</span>
                        <button class="link-btn" on:click=move |_| w.disconnect() title="Disconnect">
                            "×"
                        </button>
                    </div>
                }
                    .into_any()
            }
            None => {
                view! {
                    <button class="btn-moving" on:click=move |_| w.open()>
                        <span class="btn-moving-inner">"Connect wallet"</span>
                    </button>
                }
                    .into_any()
            }
        }}
    }
}

#[component]
pub fn WalletModal() -> impl IntoView {
    let w = use_wallet();
    view! {
        <Show when=move || w.modal.get()>
            <div class="modal-backdrop" on:click=move |_| w.modal.set(false)>
                <div class="modal" on:click=|e| e.stop_propagation()>
                    <div class="modal-head">
                        <h3>"Connect a wallet"</h3>
                        <button class="link-btn" on:click=move |_| w.modal.set(false)>"×"</button>
                    </div>
                    <p class="muted small">
                        "Blinker talks to wallets through the Wallet Standard. Switch your wallet to "
                        <b>{move || w.config.get().cluster}</b> " — transactions are submitted to that cluster."
                    </p>
                    <div class="wallet-list">
                        {move || {
                            let list = w.available.get();
                            if list.is_empty() {
                                view! {
                                    <div class="empty">
                                        <p>"No Solana wallet extension detected."</p>
                                        <p class="muted small">
                                            "Install Phantom, Solflare or Backpack, then reload."
                                        </p>
                                        <button class="btn ghost sm" on:click=move |_| w.refresh()>
                                            "Scan again"
                                        </button>
                                    </div>
                                }
                                    .into_any()
                            } else {
                                list.into_iter()
                                    .map(|wi| {
                                        let name = wi.name.clone();
                                        view! {
                                            <button
                                                class="wallet-item"
                                                disabled=move || w.connecting.get()
                                                on:click=move |_| w.connect(name.clone())
                                            >
                                                <img src=wi.icon.clone() alt="" width="28" height="28" />
                                                <span>{wi.name.clone()}</span>
                                                <span class="chev">"→"</span>
                                            </button>
                                        }
                                    })
                                    .collect_view()
                                    .into_any()
                            }
                        }}
                    </div>
                </div>
            </div>
        </Show>
    }
}
