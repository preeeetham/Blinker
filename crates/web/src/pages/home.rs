use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;

use crate::api::{self, ListResponse, StatsResponse};
use crate::fx::{Aurora, Beams, FlipWords, GridBg, Marquee, Meteors, NumberTicker, Sparkline, SpotCard, Spotlight, TextGenerate, TiltCard};
use crate::pages::card::{BlinkCard, CardData};
use crate::util::{fmt_num, poll, short};

fn demo_card() -> CardData {
    CardData {
        id: None,
        title: "Fuel the next build sprint ⚡".into(),
        description: "I'm shipping open-source Rust tooling for Solana. Every blink you send buys an hour of focused hacking.".into(),
        icon_url: "/blink.svg".into(),
        label: "Send".into(),
        unit: "SOL".into(),
        amounts: vec![0.05, 0.1, 0.5],
        allow_custom: true,
        theme: "aurora".into(),
        recipient: "5q6c2pFn3whPLUgvR5GAipoKzS2d3h9a52mAzepgaiEm".into(),
        active: true,
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    let stats = RwSignal::new(StatsResponse::default());
    let recent = RwSignal::new(ListResponse::default());
    poll(5000, move || {
        spawn_local(async move {
            if let Ok(s) = api::get::<StatsResponse>("/api/stats").await {
                stats.set(s);
            }
            if let Ok(r) = api::get::<ListResponse>("/api/blinks").await {
                recent.set(r);
            }
        })
    });

    let g = move || stats.get().global;
    let sol = Signal::derive(move || g().sol_volume_lamports.parse::<f64>().unwrap_or(0.0) / 1e9);
    let series = Signal::derive(move || stats.get().series.iter().map(|p| (p.views + p.paid * 4) as f64).collect::<Vec<_>>());
    let demo = Signal::derive(demo_card);

    view! {
        <section class="hero">
            <Aurora />
            <GridBg />
            <Spotlight />
            <Meteors count=12 />
            <div class="hero-inner">
                <div class="hero-copy">
                    <span class="eyebrow">
                        <span class="dot live"></span>
                        "Solana Actions · built in Rust, wire to pixel"
                    </span>
                    <h1 class="display">
                        "One link for" <br />
                        <FlipWords words=&["tips", "donations", "sponsors", "token drops", "payments"] />
                    </h1>
                    <TextGenerate
                        class="lead"
                        text="Describe what you're raising for, pay a 0.001 SOL activation fee, and share a blink that turns any post into a one-tap Solana payment — in SOL or any SPL token."
                    />
                    <div class="hero-ctas">
                        <A href="/create" attr:class="btn-moving lg">
                            <span class="btn-moving-inner">"Create a blink →"</span>
                        </A>
                        <A href="/system" attr:class="btn ghost lg">
                            "Watch the pipeline live"
                        </A>
                    </div>
                    <div class="hero-proof">
                        <span>"Verified on-chain"</span>
                        <span class="sep"></span>
                        <span>"Exactly-once accounting"</span>
                        <span class="sep"></span>
                        <span>"Open source"</span>
                    </div>
                </div>
                <div class="hero-visual">
                    <TiltCard>
                        <BlinkCard data=demo />
                    </TiltCard>
                    <div class="float-chip a">
                        <span class="dot live"></span>
                        "+0.1 SOL confirmed"
                    </div>
                    <div class="float-chip b mono">"logsSubscribe ✓"</div>
                </div>
            </div>
        </section>

        <section class="section">
            <div class="stat-grid">
                <SpotCard class="stat">
                    <span class="stat-label">"Live blinks"</span>
                    <span class="stat-value">
                        <NumberTicker value=Signal::derive(move || g().active_blinks as f64) />
                    </span>
                </SpotCard>
                <SpotCard class="stat">
                    <span class="stat-label">"Payments verified"</span>
                    <span class="stat-value">
                        <NumberTicker value=Signal::derive(move || g().paid as f64) />
                    </span>
                </SpotCard>
                <SpotCard class="stat">
                    <span class="stat-label">"SOL moved"</span>
                    <span class="stat-value">
                        <NumberTicker value=sol />
                    </span>
                </SpotCard>
                <SpotCard class="stat">
                    <span class="stat-label">"Pipeline events"</span>
                    <span class="stat-value">
                        <NumberTicker value=Signal::derive(move || g().events_applied as f64) />
                    </span>
                </SpotCard>
            </div>
        </section>

        <section class="section">
            <div class="section-head">
                <span class="eyebrow">"Why Blinker"</span>
                <h2 class="h2">"Everything a payment link should be."</h2>
            </div>
            <div class="bento">
                <SpotCard class="bento-item wide">
                    <div class="bento-copy">
                        <h3>"Unfurls wherever links do"</h3>
                        <p class="muted">
                            "Every blink is a spec-compliant Solana Action with an "
                            <code>"actions.json"</code>
                            " mapping, so wallets and X render it as buttons. Or share the Blinker page — it works as a standalone checkout."
                        </p>
                    </div>
                    <div class="url-morph">
                        <div class="url-bar mono">"blinker.app/b/pG2SwvqeEMJV"</div>
                        <div class="url-arrow">"↓"</div>
                        <div class="url-mini">
                            <div class="mini-img"></div>
                            <div class="mini-lines">
                                <span></span>
                                <span></span>
                            </div>
                            <div class="mini-btns">
                                <span>"0.1"</span>
                                <span>"0.5"</span>
                                <span>"1"</span>
                            </div>
                        </div>
                    </div>
                </SpotCard>
                <SpotCard class="bento-item">
                    <h3>"SOL or any SPL token"</h3>
                    <p class="muted">"Paste a mint — decimals come straight from the chain, and the recipient's token account is created on the fly."</p>
                    <div class="token-chips">
                        <span class="chip">"SOL"</span>
                        <span class="chip">"USDC"</span>
                        <span class="chip">"BONK"</span>
                        <span class="chip">"JUP"</span>
                        <span class="chip ghost">"+ any mint"</span>
                    </div>
                </SpotCard>
                <SpotCard class="bento-item beams-card">
                    <Beams />
                    <h3>"Verified on-chain, exactly once"</h3>
                    <p class="muted">"A websocket watcher, the wallet callback and a backfill sweep all race to report each payment. Three layers of dedup make sure it counts once."</p>
                </SpotCard>
                <SpotCard class="bento-item">
                    <h3>"Live analytics"</h3>
                    <p class="muted">"Views, transactions built, confirmed payments and unique payers — rebuilt from an append-only event log on every boot."</p>
                    <Sparkline values=series width=320.0 height=70.0 />
                </SpotCard>
                <SpotCard class="bento-item">
                    <h3>"Small, fast, measured"</h3>
                    <ul class="spec-list mono">
                        <li><span>"unique payers"</span><b>"1 KiB HLL/blink"</b></li>
                        <li><span>"event log"</span><b>"group-commit fsync"</b></li>
                        <li><span>"tx build"</span><b>"no RPC on hot path"</b></li>
                    </ul>
                </SpotCard>
            </div>
        </section>

        <section class="section">
            <div class="section-head">
                <span class="eyebrow">"How it works"</span>
                <h2 class="h2">"Live in under a minute."</h2>
            </div>
            <div class="steps">
                <div class="step">
                    <span class="step-num">"01"</span>
                    <h3>"Describe it"</h3>
                    <p class="muted">"Title, image, preset amounts and who gets paid. Watch the card update as you type."</p>
                </div>
                <div class="step-line"><span></span></div>
                <div class="step">
                    <span class="step-num">"02"</span>
                    <h3>"Activate"</h3>
                    <p class="muted">"Sign a 0.001 SOL fee. The watcher spots it on-chain and switches your blink on."</p>
                </div>
                <div class="step-line"><span></span></div>
                <div class="step">
                    <span class="step-num">"03"</span>
                    <h3>"Share anywhere"</h3>
                    <p class="muted">"Post the link. Every view, click and payment streams into your dashboard."</p>
                </div>
            </div>
        </section>

        <section class="section">
            <div class="section-head">
                <span class="eyebrow">"Live now"</span>
                <h2 class="h2">"Fresh blinks on this server."</h2>
            </div>
            {move || {
                let items = recent.get().items;
                if items.is_empty() {
                    view! {
                        <div class="empty-wide">
                            <p class="muted">"No active blinks yet — yours could be first."</p>
                            <A href="/create" attr:class="btn primary">"Create one"</A>
                        </div>
                    }
                        .into_any()
                } else {
                    let items = StoredValue::new(items);
                    view! {
                        <Marquee>
                            {move || {
                                items
                                    .get_value()
                                    .into_iter()
                                    .map(|it| {
                                        let b = it.blink;
                                        let paid = it.stats.map(|s| s.paid).unwrap_or(0);
                                        view! {
                                            <a class="mq-card" href=format!("/b/{}", b.id)>
                                                <img src=b.icon_url.clone() alt="" />
                                                <div>
                                                    <div class="mq-title">{b.title.clone()}</div>
                                                    <div class="muted small mono">
                                                        {short(&b.recipient)} " · " {fmt_num(paid as f64)} " payments · "
                                                        {b.unit()}
                                                    </div>
                                                </div>
                                            </a>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </Marquee>
                    }
                        .into_any()
                }
            }}
        </section>

        <section class="section">
            <div class="cta-panel">
                <Beams />
                <h2 class="h2">"Ship your first blink."</h2>
                <p class="muted">"It costs 0.001 SOL to activate. Everything after that is yours."</p>
                <A href="/create" attr:class="btn-moving lg">
                    <span class="btn-moving-inner">"Start creating"</span>
                </A>
            </div>
        </section>
    }
}
