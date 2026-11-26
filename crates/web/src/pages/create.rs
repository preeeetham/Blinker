use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use serde_json::{json, Value};

use crate::api::{self, Blink, BlinkDetail, CreateBlink, CreateResponse};
use crate::fx::{GridBg, Spotlight, TiltCard};
use crate::pages::card::{BlinkCard, CardData};
use crate::util::{copy_to_clipboard, fmt_num, toast};
use crate::wallet::{gloo_timer, use_wallet};

#[derive(Clone, PartialEq)]
enum Stage {
    Edit,
    Working(&'static str),
    Done(Box<Blink>),
}

const THEMES: &[(&str, &str)] =
    &[("aurora", "Aurora"), ("sunset", "Sunset"), ("ocean", "Ocean"), ("lime", "Lime"), ("mono", "Mono")];

#[component]
pub fn CreatePage() -> impl IntoView {
    let w = use_wallet();
    let kind = RwSignal::new("sol".to_string());
    let title = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let icon_url = RwSignal::new(String::new());
    let label = RwSignal::new("Send".to_string());
    let recipient = RwSignal::new(String::new());
    let mint = RwSignal::new(String::new());
    let symbol = RwSignal::new(String::new());
    let amounts = [RwSignal::new("0.05".to_string()), RwSignal::new("0.1".to_string()), RwSignal::new("0.5".to_string())];
    let allow_custom = RwSignal::new(true);
    let theme = RwSignal::new("aurora".to_string());
    let stage = RwSignal::new(Stage::Edit);

    // Default the recipient to the connected wallet.
    Effect::new(move |_| {
        if let Some(a) = w.address.get()
            && recipient.get_untracked().is_empty()
        {
            recipient.set(a);
        }
    });

    let parsed_amounts = move || -> Vec<f64> {
        amounts.iter().filter_map(|a| a.get().trim().parse::<f64>().ok()).filter(|a| *a > 0.0 && a.is_finite()).collect()
    };
    let unit = move || {
        if kind.get() == "sol" {
            "SOL".to_string()
        } else {
            let s = symbol.get().trim().to_uppercase();
            if s.is_empty() { "TOKEN".into() } else { s }
        }
    };

    let preview = Signal::derive(move || CardData {
        id: None,
        title: title.get(),
        description: description.get(),
        icon_url: icon_url.get(),
        label: label.get(),
        unit: unit(),
        amounts: parsed_amounts(),
        allow_custom: allow_custom.get(),
        theme: theme.get(),
        recipient: recipient.get(),
        active: false,
    });

    let submit = move |_| {
        let Some(creator) = w.address.get_untracked() else {
            w.open();
            toast(false, "Connect a wallet to pay the activation fee");
            return;
        };
        let req = CreateBlink {
            kind: kind.get_untracked(),
            title: title.get_untracked(),
            description: description.get_untracked(),
            icon_url: icon_url.get_untracked(),
            label: label.get_untracked(),
            recipient: recipient.get_untracked().trim().to_string(),
            creator,
            mint: (kind.get_untracked() == "spl").then(|| mint.get_untracked().trim().to_string()),
            token_symbol: (kind.get_untracked() == "spl").then(|| symbol.get_untracked()),
            amounts: untrack(parsed_amounts),
            allow_custom: allow_custom.get_untracked(),
            theme: theme.get_untracked(),
        };
        stage.set(Stage::Working("Creating blink…"));
        spawn_local(async move {
            let res: Result<Blink, String> = async {
                let created: CreateResponse = api::post("/api/blinks", &req).await?;
                stage.set(Stage::Working("Approve the 0.001 SOL activation fee in your wallet…"));
                let sig = w.sign_and_submit(&created.transaction).await?;
                stage.set(Stage::Working("Fee sent — waiting for the watcher to see it on-chain…"));
                let _: Value = api::post(&format!("/api/blinks/{}/confirm", created.id), &json!({ "signature": sig })).await?;
                for _ in 0..90 {
                    let d: BlinkDetail = api::get(&format!("/api/blinks/{}", created.id)).await?;
                    if d.blink.is_active() {
                        return Ok(d.blink);
                    }
                    gloo_timer(1000).await;
                }
                Err("Fee not confirmed yet. Check the dashboard in a minute.".into())
            }
            .await;
            match res {
                Ok(b) => {
                    toast(true, "Your blink is live ✨");
                    stage.set(Stage::Done(Box::new(b)));
                }
                Err(e) => {
                    toast(false, e);
                    stage.set(Stage::Edit);
                }
            }
        });
    };

    let field = move |label_text: &'static str, sig: RwSignal<String>, placeholder: &'static str, max: usize| {
        view! {
            <label class="field">
                <span class="field-label">
                    {label_text}
                    <span class="count">{move || format!("{}/{max}", sig.get().chars().count())}</span>
                </span>
                <input
                    type="text"
                    maxlength=max
                    placeholder=placeholder
                    prop:value=move || sig.get()
                    on:input=move |e| sig.set(event_target_value(&e))
                />
            </label>
        }
    };

    view! {
        <section class="page create">
            <GridBg />
            <Spotlight fill="#a855f7" />
            <div class="page-head">
                <span class="eyebrow">"New blink"</span>
                <h1 class="h1">"Design a payment link"</h1>
                <p class="muted">"Everything on the right updates as you type — that's exactly how wallets will render it."</p>
            </div>

            {move || match stage.get() {
                Stage::Done(b) => view! { <Success blink=*b /> }.into_any(),
                st => {
                    let working = matches!(st, Stage::Working(_));
                    let msg = if let Stage::Working(m) = st { m } else { "" };
                    view! {
                        <div class="create-grid">
                            <form class="panel form" on:submit=|e| e.prevent_default()>
                                <div class="seg">
                                    <button
                                        type="button"
                                        class=move || if kind.get() == "sol" { "seg-btn on" } else { "seg-btn" }
                                        on:click=move |_| kind.set("sol".into())
                                    >
                                        "◎ SOL"
                                    </button>
                                    <button
                                        type="button"
                                        class=move || if kind.get() == "spl" { "seg-btn on" } else { "seg-btn" }
                                        on:click=move |_| kind.set("spl".into())
                                    >
                                        "SPL token"
                                    </button>
                                </div>

                                {field("Title", title, "Buy me a coffee ☕", 80)}
                                <label class="field">
                                    <span class="field-label">
                                        "Description"
                                        <span class="count">{move || format!("{}/400", description.get().chars().count())}</span>
                                    </span>
                                    <textarea
                                        rows="3"
                                        maxlength="400"
                                        placeholder="Tell people what their support goes toward."
                                        prop:value=move || description.get()
                                        on:input=move |e| description.set(event_target_value(&e))
                                    ></textarea>
                                </label>
                                {field("Image URL (https)", icon_url, "https://… (optional — defaults to the Blinker mark)", 500)}

                                <Show when=move || kind.get() == "spl">
                                    <div class="row2">
                                        {field("Token mint", mint, "Mint address", 44)}
                                        {field("Symbol", symbol, "USDC", 10)}
                                    </div>
                                </Show>

                                <div class="field">
                                    <span class="field-label">{move || format!("Preset amounts ({})", unit())}</span>
                                    <div class="amounts">
                                        {amounts
                                            .iter()
                                            .map(|a| {
                                                let a = *a;
                                                view! {
                                                    <input
                                                        type="number"
                                                        min="0"
                                                        step="any"
                                                        prop:value=move || a.get()
                                                        on:input=move |e| a.set(event_target_value(&e))
                                                    />
                                                }
                                            })
                                            .collect_view()}
                                    </div>
                                </div>

                                <div class="row2">
                                    {field("Button label", label, "Send", 32)}
                                    <label class="toggle">
                                        <input
                                            type="checkbox"
                                            prop:checked=move || allow_custom.get()
                                            on:change=move |e| allow_custom.set(event_target_checked(&e))
                                        />
                                        <span class="toggle-ui"></span>
                                        "Allow custom amount"
                                    </label>
                                </div>

                                {field("Recipient wallet", recipient, "Who receives the funds", 44)}

                                <div class="field">
                                    <span class="field-label">"Theme"</span>
                                    <div class="swatches">
                                        {THEMES
                                            .iter()
                                            .map(|(id, name)| {
                                                let id = *id;
                                                view! {
                                                    <button
                                                        type="button"
                                                        title=*name
                                                        class=move || {
                                                            format!("swatch theme-{id} {}", if theme.get() == id { "on" } else { "" })
                                                        }
                                                        on:click=move |_| theme.set(id.into())
                                                    ></button>
                                                }
                                            })
                                            .collect_view()}
                                    </div>
                                </div>

                                <div class="form-foot">
                                    <div class="fee-note">
                                        <span class="muted small">"Activation fee"</span>
                                        <b>{move || format!("{} SOL", fmt_num(w.config.get().fee_sol))}</b>
                                        <span class="muted small">{move || format!("on {}", w.config.get().cluster)}</span>
                                    </div>
                                    <button type="button" class="btn-moving lg" disabled=working on:click=submit>
                                        <span class="btn-moving-inner">
                                            {move || if w.address.get().is_some() { "Create & activate" } else { "Connect wallet to create" }}
                                        </span>
                                    </button>
                                </div>
                                <Show when=move || working>
                                    <div class="progress">
                                        <span class="spinner"></span>
                                        {msg}
                                    </div>
                                </Show>
                            </form>

                            <div class="preview-col">
                                <span class="preview-label">"Live preview"</span>
                                <TiltCard>
                                    <BlinkCard data=preview />
                                </TiltCard>
                            </div>
                        </div>
                    }
                        .into_any()
                }
            }}
        </section>
    }
}

#[component]
fn Success(blink: Blink) -> impl IntoView {
    let tweet = format!(
        "https://twitter.com/intent/tweet?text={}&url={}",
        "Support%20me%20with%20a%20Solana%20blink%20%E2%9A%A1",
        blink.blink_url.replace(':', "%3A").replace('/', "%2F")
    );
    let rows = [("Blink page", blink.blink_url.clone()), ("Action URL", blink.action_url.clone()), ("dial.to", blink.dial_url.clone())];
    let card = Signal::derive({
        let b = blink.clone();
        move || CardData::from(&b)
    });
    view! {
        <div class="success">
            <div class="panel success-panel">
                <div class="success-badge">"✓"</div>
                <h2 class="h2">"Your blink is live"</h2>
                <p class="muted">"The activation fee was verified on-chain. Share any of these links:"</p>
                <div class="link-rows">
                    {rows
                        .into_iter()
                        .map(|(k, v)| {
                            let v2 = v.clone();
                            view! {
                                <div class="link-row">
                                    <span class="muted small">{k}</span>
                                    <code class="mono">{v.clone()}</code>
                                    <button
                                        class="btn ghost sm"
                                        on:click=move |_| {
                                            copy_to_clipboard(&v2);
                                            toast(true, "Copied");
                                        }
                                    >
                                        "Copy"
                                    </button>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
                <div class="hero-ctas">
                    <A href=format!("/b/{}", blink.id) attr:class="btn primary">"Open blink"</A>
                    <a class="btn ghost" href=tweet target="_blank" rel="noreferrer">"Share on X"</a>
                    <A href="/dashboard" attr:class="btn ghost">"Dashboard"</A>
                </div>
                {blink
                    .fee_explorer_url
                    .clone()
                    .map(|u| view! { <a class="small link" href=u target="_blank" rel="noreferrer">"View fee transaction ↗"</a> })}
            </div>
            <TiltCard>
                <BlinkCard data=card />
            </TiltCard>
        </div>
    }
}
