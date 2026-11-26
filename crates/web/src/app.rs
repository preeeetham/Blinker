use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, MetaTags, Stylesheet, Title};
use leptos_router::components::{Route, Router, Routes, A};
use leptos_router::path;

use crate::pages::{blink::BlinkPage, create::CreatePage, dashboard::DashboardPage, home::HomePage, system::SystemPage};
use crate::util::{ToastHost, Toasts};
use crate::wallet::{use_wallet, Wallet, WalletButton, WalletModal};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" class="dark">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="theme-color" content="#06060b" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(Toasts(RwSignal::new(vec![])));
    Wallet::provide();

    view! {
        <Stylesheet id="leptos" href="/pkg/blinker.css" />
        <Link rel="icon" href="/favicon.svg" />
        <Link rel="preconnect" href="https://fonts.googleapis.com" />
        <Link
            rel="stylesheet"
            href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;600&display=swap"
        />
        <Title text="Blinker — shareable Solana payment links" />
        <Router>
            <Nav />
            <main>
                <Routes fallback=|| view! { <NotFound /> }>
                    <Route path=path!("/") view=HomePage />
                    <Route path=path!("/create") view=CreatePage />
                    <Route path=path!("/dashboard") view=DashboardPage />
                    <Route path=path!("/system") view=SystemPage />
                    <Route path=path!("/b/:id") view=BlinkPage />
                </Routes>
            </main>
            <Footer />
            <WalletModal />
            <ToastHost />
        </Router>
    }
}

#[component]
fn Nav() -> impl IntoView {
    let w = use_wallet();
    view! {
        <header class="nav-wrap">
            <nav class="nav">
                <A href="/" attr:class="brand">
                    <span class="brand-mark">
                        <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true">
                            <path d="M13 2 4 14h7l-1 8 9-12h-7l1-8Z" fill="currentColor"></path>
                        </svg>
                    </span>
                    "Blinker"
                </A>
                <div class="nav-links">
                    <A href="/create">"Create"</A>
                    <A href="/dashboard">"Dashboard"</A>
                    <A href="/system">"System"</A>
                </div>
                <div class="nav-right">
                    <span class="cluster-pill" title="Solana cluster this server is connected to">
                        <span class="dot live"></span>
                        {move || {
                            let c = w.config.get().cluster;
                            if c.is_empty() { "…".to_string() } else { c }
                        }}
                    </span>
                    <WalletButton />
                </div>
            </nav>
        </header>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="footer-inner">
                <div>
                    <div class="brand small">"Blinker"</div>
                    <p class="muted small">
                        "Rust end to end: Axum + Tokio backend, Leptos/WASM UI, hand-rolled Solana wire format."
                    </p>
                </div>
                <div class="footer-links">
                    <a href="/actions.json">"actions.json"</a>
                    <a href="/metrics">"/metrics"</a>
                    <a href="https://github.com/preeeetham/Blinker" target="_blank" rel="noreferrer">
                        "GitHub"
                    </a>
                </div>
            </div>
        </footer>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <section class="page narrow center">
            <h1 class="display">"404"</h1>
            <p class="muted">"That page blinked out of existence."</p>
            <A href="/" attr:class="btn primary">"Go home"</A>
        </section>
    }
}
