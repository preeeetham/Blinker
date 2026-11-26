use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::api::{self, BlinkDetail, SeriesResponse};
use crate::fx::{Aurora, Bars, GridBg, NumberTicker, SpotCard, TiltCard};
use crate::pages::card::{BlinkCard, CardData};
use crate::util::{copy_to_clipboard, fmt_num, poll, short, toast};

#[component]
pub fn BlinkPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();
    let detail = RwSignal::new(None::<BlinkDetail>);
    let missing = RwSignal::new(false);
    let series = RwSignal::new(SeriesResponse::default());

    let load = move || {
        let id = id();
        spawn_local(async move {
            match api::get::<BlinkDetail>(&format!("/api/blinks/{id}")).await {
                Ok(d) => detail.set(Some(d)),
                Err(_) => missing.set(true),
            }
            if let Ok(s) = api::get::<SeriesResponse>(&format!("/api/blinks/{id}/series?res=minute")).await {
                series.set(s);
            }
        });
    };
    poll(4000, move || untrack(load));

    let card = Signal::derive(move || detail.get().map(|d| CardData::from(&d.blink)).unwrap_or_default());
    let stats = move || detail.get().and_then(|d| d.stats).unwrap_or_default();
    let unit = move || detail.get().map(|d| d.blink.unit()).unwrap_or_default();
    let volume = Signal::derive(move || detail.get().map(|d| d.blink.ui_volume(&stats().volume)).unwrap_or(0.0));
    let bars = Signal::derive(move || series.get().points.iter().map(|p| (p.views as f64, p.paid as f64)).collect::<Vec<_>>());

    view! {
        <Title text=move || detail.get().map(|d| format!("{} · Blinker", d.blink.title)).unwrap_or_else(|| "Blink · Blinker".into()) />
        <section class="page blink-page">
            <Aurora />
            <GridBg />
            {move || {
                if missing.get() {
                    return view! {
                        <div class="narrow center">
                            <h1 class="h1">"Blink not found"</h1>
                            <p class="muted">"This link doesn't point to a blink on this server."</p>
                        </div>
                    }
                        .into_any();
                }
                let Some(d) = detail.get() else {
                    return view! { <div class="narrow center"><span class="spinner lg"></span></div> }.into_any();
                };
                let b = d.blink.clone();
                let link = b.blink_url.clone();
                let action = b.action_url.clone();
                view! {
                    <div class="blink-layout">
                        <div class="blink-main">
                            <TiltCard>
                                <BlinkCard data=card interactive=true on_paid=Callback::new(move |_| load()) />
                            </TiltCard>
                            <p class="muted small center">
                                "Payments go straight to "
                                <span class="mono">{short(&b.recipient)}</span>
                                ". Blinker never holds funds."
                            </p>
                        </div>
                        <div class="blink-side">
                            <div class="stat-grid tight">
                                <SpotCard class="stat">
                                    <span class="stat-label">"Views"</span>
                                    <span class="stat-value"><NumberTicker value=Signal::derive(move || stats().views as f64) /></span>
                                </SpotCard>
                                <SpotCard class="stat">
                                    <span class="stat-label">"Payments"</span>
                                    <span class="stat-value"><NumberTicker value=Signal::derive(move || stats().paid as f64) /></span>
                                </SpotCard>
                                <SpotCard class="stat">
                                    <span class="stat-label">{move || format!("Raised ({})", unit())}</span>
                                    <span class="stat-value"><NumberTicker value=volume /></span>
                                </SpotCard>
                                <SpotCard class="stat">
                                    <span class="stat-label">"Unique payers"</span>
                                    <span class="stat-value"><NumberTicker value=Signal::derive(move || stats().unique_payers as f64) /></span>
                                </SpotCard>
                            </div>
                            <div class="panel">
                                <div class="panel-head">
                                    <h3>"Last hour"</h3>
                                    <span class="legend"><i class="lg-a"></i>"views"<i class="lg-b"></i>"payments"</span>
                                </div>
                                <Bars values=bars width=600.0 height=120.0 />
                            </div>
                            <div class="panel">
                                <h3>"Share"</h3>
                                <div class="link-rows">
                                    <div class="link-row">
                                        <span class="muted small">"Page"</span>
                                        <code class="mono">{link.clone()}</code>
                                        <button class="btn ghost sm" on:click=move |_| { copy_to_clipboard(&link); toast(true, "Copied"); }>"Copy"</button>
                                    </div>
                                    <div class="link-row">
                                        <span class="muted small">"Action"</span>
                                        <code class="mono">{action.clone()}</code>
                                        <button class="btn ghost sm" on:click=move |_| { copy_to_clipboard(&action); toast(true, "Copied"); }>"Copy"</button>
                                    </div>
                                </div>
                            </div>
                            <div class="panel">
                                <h3>"Recent payments"</h3>
                                {if d.payments.is_empty() {
                                    view! { <p class="muted small">"No payments yet. Be the first."</p> }.into_any()
                                } else {
                                    let u = b.unit();
                                    view! {
                                        <ul class="tx-list">
                                            {d.payments
                                                .into_iter()
                                                .map(|p| {
                                                    view! {
                                                        <li>
                                                            <span class="mono">{short(&p.payer)}</span>
                                                            <b>{p.amount.clone()} " " {u.clone()}</b>
                                                            <a href=p.explorer_url target="_blank" rel="noreferrer" class="link small">
                                                                {short(&p.signature)} " ↗"
                                                            </a>
                                                        </li>
                                                    }
                                                })
                                                .collect_view()}
                                        </ul>
                                    }
                                        .into_any()
                                }}
                                <p class="muted small">{format!("{} conversion from view to payment", fmt_num((stats().conversion * 1000.0).round() / 10.0) + "%")}</p>
                            </div>
                        </div>
                    </div>
                }
                    .into_any()
            }}
        </section>
    }
}
