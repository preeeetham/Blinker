//! The blink card: how a blink unfurls in a wallet / on X. Used as the live
//! preview while creating, as the hero demo, and as the working payment UI
//! on `/b/:id` (where `interactive` is set).

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json::{json, Value};

use crate::api;
use crate::util::{fmt_num, short, toast};
use crate::wallet::use_wallet;

#[derive(Clone, PartialEq, Default)]
pub struct CardData {
    pub id: Option<String>,
    pub title: String,
    pub description: String,
    pub icon_url: String,
    pub label: String,
    pub unit: String,
    pub amounts: Vec<f64>,
    pub allow_custom: bool,
    pub theme: String,
    pub recipient: String,
    pub active: bool,
}

impl From<&api::Blink> for CardData {
    fn from(b: &api::Blink) -> Self {
        Self {
            id: Some(b.id.clone()),
            title: b.title.clone(),
            description: b.description.clone(),
            icon_url: b.icon_url.clone(),
            label: b.label.clone(),
            unit: b.unit(),
            amounts: b.amounts.clone(),
            allow_custom: b.allow_custom,
            theme: b.theme.clone(),
            recipient: b.recipient.clone(),
            active: b.is_active(),
        }
    }
}

#[component]
pub fn BlinkCard(
    #[prop(into)] data: Signal<CardData>,
    #[prop(default = false)] interactive: bool,
    /// Called with the signature after a successful payment.
    #[prop(optional)]
    on_paid: Option<Callback<String>>,
) -> impl IntoView {
    let w = use_wallet();
    let busy = RwSignal::new(None::<String>);
    let custom = RwSignal::new(String::new());
    let last_sig = RwSignal::new(None::<String>);

    let pay = move |amount: f64| {
        if !interactive {
            return;
        }
        let d = data.get_untracked();
        let Some(id) = d.id.clone() else { return };
        let Some(account) = w.address.get_untracked() else {
            w.open();
            return;
        };
        if !(amount.is_finite() && amount > 0.0) {
            toast(false, "Enter an amount greater than zero");
            return;
        }
        busy.set(Some(fmt_num(amount)));
        spawn_local(async move {
            let res: Result<String, String> = async {
                let r: Value = api::post(&format!("/api/actions/{id}?amount={amount}"), &json!({ "account": account })).await?;
                let tx = r["transaction"].as_str().ok_or("no transaction in action response")?;
                let sig = w.sign_and_submit(tx).await?;
                if let Some(next) = r["links"]["next"]["href"].as_str() {
                    let _: Result<Value, String> =
                        api::post(next, &json!({ "account": account, "signature": sig })).await;
                }
                Ok(sig)
            }
            .await;
            busy.set(None);
            match res {
                Ok(sig) => {
                    toast(true, format!("Sent {} {} ✓", fmt_num(amount), d.unit));
                    last_sig.set(Some(sig.clone()));
                    if let Some(cb) = on_paid {
                        cb.run(sig);
                    }
                }
                Err(e) => toast(false, e),
            }
        });
    };

    view! {
        <article class=move || format!("blink-card theme-{}", { let t = data.get().theme; if t.is_empty() { "aurora".into() } else { t } })>
            <div class="blink-media">
                {move || {
                    let icon = data.get().icon_url;
                    if icon.is_empty() {
                        view! {
                            <div class="blink-media-fallback">
                                <svg viewBox="0 0 24 24" width="56" height="56" aria-hidden="true">
                                    <path d="M13 2 4 14h7l-1 8 9-12h-7l1-8Z" fill="currentColor"></path>
                                </svg>
                            </div>
                        }
                            .into_any()
                    } else {
                        view! { <img src=icon alt="" loading="lazy" /> }.into_any()
                    }
                }}
                <div class="blink-media-sheen"></div>
            </div>
            <div class="blink-body">
                <div class="blink-meta">
                    <span class="mono">{move || if data.get().recipient.is_empty() { "blinker".to_string() } else { short(&data.get().recipient) }}</span>
                    {move || {
                        if data.get().active {
                            view! { <span class="badge ok">"verified"</span> }.into_any()
                        } else if data.get().id.is_some() {
                            view! { <span class="badge warn">"awaiting fee"</span> }.into_any()
                        } else {
                            view! { <span class="badge">"preview"</span> }.into_any()
                        }
                    }}
                </div>
                <h3 class="blink-title">
                    {move || { let t = data.get().title; if t.is_empty() { "Your blink title".into() } else { t } }}
                </h3>
                <p class="blink-desc">
                    {move || {
                        let t = data.get().description;
                        if t.is_empty() { "Describe what people are supporting.".into() } else { t }
                    }}
                </p>
                <div class="blink-actions">
                    {move || {
                        let d = data.get();
                        d.amounts
                            .iter()
                            .copied()
                            .map(|a| {
                                let label = format!("{} {}", fmt_num(a), d.unit);
                                view! {
                                    <button
                                        class="blink-btn"
                                        disabled=move || busy.get().is_some() || (interactive && !data.get().active)
                                        on:click=move |_| pay(a)
                                    >
                                        {move || if busy.get() == Some(fmt_num(a)) { "Signing…".to_string() } else { label.clone() }}
                                    </button>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <Show when=move || data.get().allow_custom>
                    <div class="blink-custom">
                        <input
                            type="number"
                            min="0"
                            step="any"
                            placeholder=move || format!("Enter a {} amount", data.get().unit)
                            prop:value=move || custom.get()
                            on:input=move |e| custom.set(event_target_value(&e))
                        />
                        <button
                            class="blink-btn primary"
                            disabled=move || busy.get().is_some() || (interactive && !data.get().active)
                            on:click=move |_| pay(custom.get_untracked().trim().parse().unwrap_or(0.0))
                        >
                            {move || { let l = data.get().label; if l.is_empty() { "Send".into() } else { l } }}
                        </button>
                    </div>
                </Show>
                {move || {
                    last_sig
                        .get()
                        .map(|s| {
                            view! {
                                <p class="blink-sent">
                                    <span class="dot live"></span>
                                    "Submitted " <span class="mono">{short(&s)}</span>
                                    " — confirming on-chain"
                                </p>
                            }
                        })
                }}
            </div>
        </article>
    }
}
