use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;

use crate::api::{self, BlinkWithStats, ListResponse, SeriesResponse};
use crate::fx::{Bars, GridBg, Sparkline, SpotCard};
use crate::util::{fmt_num, poll, short};
use crate::wallet::use_wallet;

#[component]
pub fn DashboardPage() -> impl IntoView {
    let w = use_wallet();
    let items = RwSignal::new(Vec::<BlinkWithStats>::new());
    let loaded = RwSignal::new(false);
    let selected = RwSignal::new(None::<String>);
    let series = RwSignal::new(SeriesResponse::default());
    let hourly = RwSignal::new(false);

    poll(5000, move || {
        let Some(addr) = w.address.get_untracked() else { return };
        let sel = selected.get_untracked();
        let hr = hourly.get_untracked();
        spawn_local(async move {
            if let Ok(r) = api::get::<ListResponse>(&format!("/api/blinks?creator={addr}")).await {
                if sel.is_none()
                    && let Some(first) = r.items.first()
                {
                    selected.set(Some(first.blink.id.clone()));
                }
                items.set(r.items);
            }
            loaded.set(true);
            if let Some(id) = selected.get_untracked() {
                let res = if hr { "hour" } else { "minute" };
                if let Ok(s) = api::get::<SeriesResponse>(&format!("/api/blinks/{id}/series?res={res}")).await {
                    series.set(s);
                }
            }
        });
    });
    // Refetch immediately when the wallet connects or the selection changes.
    Effect::new(move |_| {
        let (_a, _s, _h) = (w.address.get(), selected.get(), hourly.get());
        let Some(addr) = w.address.get_untracked() else { return };
        let id = selected.get_untracked();
        let hr = hourly.get_untracked();
        spawn_local(async move {
            if let Ok(r) = api::get::<ListResponse>(&format!("/api/blinks?creator={addr}")).await {
                items.set(r.items);
            }
            loaded.set(true);
            if let Some(id) = id {
                let res = if hr { "hour" } else { "minute" };
                if let Ok(s) = api::get::<SeriesResponse>(&format!("/api/blinks/{id}/series?res={res}")).await {
                    series.set(s);
                }
            }
        });
    });

    let totals = move || {
        items.get().iter().fold((0u64, 0u64, 0u64), |acc, it| {
            let s = it.stats.clone().unwrap_or_default();
            (acc.0 + s.views, acc.1 + s.posts, acc.2 + s.paid)
        })
    };
    let bars = Signal::derive(move || series.get().points.iter().map(|p| (p.views as f64, p.paid as f64)).collect::<Vec<_>>());
    let sel_item = move || {
        let id = selected.get()?;
        items.get().into_iter().find(|i| i.blink.id == id)
    };

    view! {
        <section class="page dashboard">
            <GridBg />
            <div class="page-head row-between">
                <div>
                    <span class="eyebrow">"Dashboard"</span>
                    <h1 class="h1">"Your blinks"</h1>
                </div>
                <A href="/create" attr:class="btn primary">"+ New blink"</A>
            </div>

            {move || {
                if w.address.get().is_none() {
                    return view! {
                        <div class="panel empty-wide">
                            <h3>"Connect your wallet"</h3>
                            <p class="muted">"Your dashboard lists blinks created by the connected wallet."</p>
                            <button class="btn-moving" on:click=move |_| w.open()>
                                <span class="btn-moving-inner">"Connect wallet"</span>
                            </button>
                        </div>
                    }
                        .into_any();
                }
                if loaded.get() && items.get().is_empty() {
                    return view! {
                        <div class="panel empty-wide">
                            <h3>"No blinks yet"</h3>
                            <p class="muted">"Create one and its live analytics will appear here."</p>
                            <A href="/create" attr:class="btn primary">"Create a blink"</A>
                        </div>
                    }
                        .into_any();
                }
                let (v, p, paid) = totals();
                view! {
                    <div class="stat-grid">
                        <SpotCard class="stat"><span class="stat-label">"Blinks"</span><span class="stat-value">{items.get().len()}</span></SpotCard>
                        <SpotCard class="stat"><span class="stat-label">"Views"</span><span class="stat-value">{fmt_num(v as f64)}</span></SpotCard>
                        <SpotCard class="stat"><span class="stat-label">"Transactions built"</span><span class="stat-value">{fmt_num(p as f64)}</span></SpotCard>
                        <SpotCard class="stat"><span class="stat-label">"Confirmed payments"</span><span class="stat-value">{fmt_num(paid as f64)}</span></SpotCard>
                    </div>
                    <div class="dash-grid">
                        <div class="dash-list">
                            {items
                                .get()
                                .into_iter()
                                .map(|it| {
                                    let b = it.blink.clone();
                                    let s = it.stats.clone().unwrap_or_default();
                                    let id = b.id.clone();
                                    let id2 = b.id.clone();
                                    let vol = b.ui_volume(&s.volume);
                                    view! {
                                        <button
                                            class=move || if selected.get().as_deref() == Some(id.as_str()) { "dash-item on" } else { "dash-item" }
                                            on:click=move |_| selected.set(Some(id2.clone()))
                                        >
                                            <img src=b.icon_url.clone() alt="" />
                                            <div class="dash-item-body">
                                                <div class="row-between">
                                                    <b>{b.title.clone()}</b>
                                                    {if b.is_active() {
                                                        view! { <span class="badge ok">"live"</span> }.into_any()
                                                    } else {
                                                        view! { <span class="badge warn">"pending fee"</span> }.into_any()
                                                    }}
                                                </div>
                                                <span class="muted small mono">
                                                    {fmt_num(s.views as f64)} " views · " {fmt_num(s.paid as f64)} " paid · "
                                                    {fmt_num(vol)} " " {b.unit()}
                                                </span>
                                            </div>
                                        </button>
                                    }
                                })
                                .collect_view()}
                        </div>
                        <div class="panel dash-detail">
                            {move || {
                                sel_item()
                                    .map(|it| {
                                        let b = it.blink;
                                        let s = it.stats.unwrap_or_default();
                                        let spark = Signal::derive(move || {
                                            series.get().points.iter().map(|p| p.views as f64).collect::<Vec<_>>()
                                        });
                                        view! {
                                            <div class="row-between">
                                                <div>
                                                    <h3>{b.title.clone()}</h3>
                                                    <span class="muted small mono">{b.id.clone()} " · to " {short(&b.recipient)}</span>
                                                </div>
                                                <div class="seg sm">
                                                    <button class=move || if !hourly.get() { "seg-btn on" } else { "seg-btn" } on:click=move |_| hourly.set(false)>"60 min"</button>
                                                    <button class=move || if hourly.get() { "seg-btn on" } else { "seg-btn" } on:click=move |_| hourly.set(true)>"7 days"</button>
                                                </div>
                                            </div>
                                            <Bars values=bars width=640.0 height=170.0 />
                                            <span class="legend"><i class="lg-a"></i>"views"<i class="lg-b"></i>"payments"</span>
                                            <div class="kv-grid">
                                                <div><span class="muted small">"Conversion"</span><b>{format!("{:.1}%", s.conversion * 100.0)}</b></div>
                                                <div><span class="muted small">"Unique payers"</span><b>{s.unique_payers}</b></div>
                                                <div><span class="muted small">"Views (24h)"</span><b>{s.views_24h}</b></div>
                                                <div><span class="muted small">"Paid (24h)"</span><b>{s.paid_24h}</b></div>
                                            </div>
                                            <Sparkline values=spark width=640.0 height=48.0 />
                                            <div class="hero-ctas">
                                                <A href=format!("/b/{}", b.id) attr:class="btn primary sm">"Open blink"</A>
                                                <a href=b.dial_url.clone() target="_blank" rel="noreferrer" class="btn ghost sm">"dial.to ↗"</a>
                                            </div>
                                        }
                                    })
                            }}
                        </div>
                    </div>
                }
                    .into_any()
            }}
        </section>
    }
}
