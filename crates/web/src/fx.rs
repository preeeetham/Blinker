//! Visual effects inspired by Aceternity UI, re-implemented in Rust + CSS.
//!
//! Aceternity ships React/TSX components built on framer-motion. None of
//! that runs here, so each effect is rebuilt from first principles: CSS
//! keyframes for anything time-based, and Leptos signals + pointer events
//! for anything interactive (tilt, spotlight). No JS animation library.
//!
//! Everything that renders on the server is deterministic (no RNG, no
//! clocks) so hydration always matches.

use leptos::html;
use leptos::prelude::*;

/// Animated aurora backdrop (Aceternity "Aurora Background").
#[component]
pub fn Aurora() -> impl IntoView {
    view! {
        <div class="aurora" aria-hidden="true">
            <div class="aurora-a"></div>
            <div class="aurora-b"></div>
        </div>
    }
}

/// Soft spotlight cone that fades in from the top-left ("Spotlight").
#[component]
pub fn Spotlight(#[prop(default = "white")] fill: &'static str) -> impl IntoView {
    view! {
        <svg class="spotlight" viewBox="0 0 3787 2842" fill="none" aria-hidden="true">
            <g filter="url(#spot-blur)">
                <ellipse
                    cx="1924.71"
                    cy="273.501"
                    rx="1924.71"
                    ry="273.501"
                    transform="matrix(-0.822377 -0.568943 -0.568943 0.822377 3631.88 2291.09)"
                    fill=fill
                    fill-opacity="0.21"
                ></ellipse>
            </g>
            <defs>
                <filter id="spot-blur" x="0.86" y="0.84" width="3785.16" height="2840.26" filterUnits="userSpaceOnUse">
                    <feGaussianBlur stdDeviation="151" result="blur"></feGaussianBlur>
                </filter>
            </defs>
        </svg>
    }
}

/// Faint grid that fades out radially ("Grid Background").
#[component]
pub fn GridBg() -> impl IntoView {
    view! { <div class="grid-bg" aria-hidden="true"></div> }
}

/// SVG beams with a travelling gradient ("Background Beams").
#[component]
pub fn Beams() -> impl IntoView {
    let paths = (0..14)
        .map(|i| {
            let o = i as f64 * 28.0;
            format!(
                "M{:.0} {:.0}C{:.0} {:.0} {:.0} {:.0} {:.0} {:.0}C{:.0} {:.0} {:.0} {:.0} {:.0} {:.0}",
                -380.0 + o, -189.0 - o * 0.4,
                -380.0 + o, -189.0 - o * 0.4,
                -312.0 + o, 216.0 - o * 0.4,
                152.0 + o, 343.0 - o * 0.4,
                616.0 + o, 470.0 - o * 0.4,
                684.0 + o, 875.0 - o * 0.4,
                684.0 + o, 875.0 - o * 0.4
            )
        })
        .collect::<Vec<_>>();
    view! {
        <svg class="beams" viewBox="0 0 696 316" fill="none" aria-hidden="true">
            <defs>
                <linearGradient id="beam-grad" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="#18CCFC" stop-opacity="0"></stop>
                    <stop offset="35%" stop-color="#18CCFC"></stop>
                    <stop offset="65%" stop-color="#6344F5"></stop>
                    <stop offset="100%" stop-color="#AE48FF" stop-opacity="0"></stop>
                </linearGradient>
            </defs>
            {paths
                .into_iter()
                .enumerate()
                .map(|(i, d)| {
                    view! {
                        <path d=d.clone() class="beam-base"></path>
                        <path
                            d=d
                            class="beam-glow"
                            style=format!("animation-delay:{:.1}s;animation-duration:{:.1}s", i as f64 * 0.7, 7.0 + (i % 5) as f64)
                        ></path>
                    }
                })
                .collect_view()}
        </svg>
    }
}

/// Falling streaks ("Meteors"). Positions are derived from the index so the
/// server and client render identical markup.
#[component]
pub fn Meteors(#[prop(default = 14)] count: usize) -> impl IntoView {
    view! {
        <div class="meteors" aria-hidden="true">
            {(0..count)
                .map(|i| {
                    let left = (i * 37 % 100) as f64;
                    let delay = (i * 13 % 7) as f64 * 0.6;
                    let dur = 4.0 + (i * 7 % 5) as f64;
                    view! {
                        <span
                            class="meteor"
                            style=format!("left:{left}%;animation-delay:{delay:.1}s;animation-duration:{dur:.1}s")
                        ></span>
                    }
                })
                .collect_view()}
        </div>
    }
}

/// Words fade in one after another ("Text Generate Effect").
#[component]
pub fn TextGenerate(text: &'static str, #[prop(default = "")] class: &'static str) -> impl IntoView {
    view! {
        <p class=format!("textgen {class}")>
            {text
                .split(' ')
                .enumerate()
                .map(|(i, w)| {
                    view! { <span style=format!("animation-delay:{}ms", 60 * i)>{w}</span> " " }
                })
                .collect_view()}
        </p>
    }
}

/// Cycles through words with a blur/slide transition ("Flip Words").
#[component]
pub fn FlipWords(words: &'static [&'static str]) -> impl IntoView {
    let idx = RwSignal::new(0usize);
    // Effects only run in the browser, so the timer never exists during SSR.
    Effect::new(move |_| {
        if let Ok(h) = set_interval_with_handle(
            move || idx.update(|i| *i = (*i + 1) % words.len()),
            std::time::Duration::from_millis(2400),
        ) {
            on_cleanup(move || h.clear());
        }
    });
    view! {
        <span class="flip">
            {move || {
                let i = idx.get();
                view! { <span class="flip-word gradient-text">{words[i]}</span> }
            }}
        </span>
    }
}

/// Counts up to `value` with an ease-out curve ("Number Ticker").
#[component]
pub fn NumberTicker(
    #[prop(into)] value: Signal<f64>,
    #[prop(default = crate::util::fmt_num)] format: fn(f64) -> String,
) -> impl IntoView {
    let shown = RwSignal::new(0.0f64);
    Effect::new(move |prev: Option<f64>| {
        let target = value.get();
        let from = prev.unwrap_or(0.0);
        let start = js_sys::Date::now();
        let handle = std::rc::Rc::new(std::cell::Cell::new(None::<IntervalHandle>));
        let h2 = handle.clone();
        if let Ok(h) = set_interval_with_handle(
            move || {
                let t = ((js_sys::Date::now() - start) / 900.0).min(1.0);
                let eased = 1.0 - (1.0 - t).powi(3);
                shown.set(from + (target - from) * eased);
                if t >= 1.0
                    && let Some(h) = h2.take()
                {
                    h.clear();
                }
            },
            std::time::Duration::from_millis(16),
        ) {
            handle.set(Some(h));
        }
        target
    });
    view! { <span class="ticker">{move || format(shown.get())}</span> }
}

/// Card that tilts toward the pointer with depth ("3D Card Effect").
#[component]
pub fn TiltCard(children: Children, #[prop(default = "")] class: &'static str) -> impl IntoView {
    let el: NodeRef<html::Div> = NodeRef::new();
    let transform = RwSignal::new(String::from("rotateX(0deg) rotateY(0deg)"));
    let glare = RwSignal::new(String::from("50% 50%"));
    let on_move = move |ev: leptos::ev::MouseEvent| {
        if let Some(div) = el.get() {
            let r = div.get_bounding_client_rect();
            let x = (ev.client_x() as f64 - r.left()) / r.width();
            let y = (ev.client_y() as f64 - r.top()) / r.height();
            transform.set(format!("rotateX({:.2}deg) rotateY({:.2}deg)", (0.5 - y) * 14.0, (x - 0.5) * 18.0));
            glare.set(format!("{:.0}% {:.0}%", x * 100.0, y * 100.0));
        }
    };
    view! {
        <div class=format!("tilt-wrap {class}")>
            <div
                class="tilt"
                node_ref=el
                on:mousemove=on_move
                on:mouseleave=move |_| transform.set("rotateX(0deg) rotateY(0deg)".into())
                style:transform=move || transform.get()
                style=("--glare", move || glare.get())
            >
                {children()}
                <div class="tilt-glare"></div>
            </div>
        </div>
    }
}

/// Card with a radial glow that follows the cursor ("Card Spotlight").
#[component]
pub fn SpotCard(children: Children, #[prop(default = "")] class: &'static str) -> impl IntoView {
    let el: NodeRef<html::Div> = NodeRef::new();
    let pos = RwSignal::new((0.0f64, 0.0f64));
    let on_move = move |ev: leptos::ev::MouseEvent| {
        if let Some(div) = el.get() {
            let r = div.get_bounding_client_rect();
            pos.set((ev.client_x() as f64 - r.left(), ev.client_y() as f64 - r.top()));
        }
    };
    view! {
        <div
            class=format!("spot-card {class}")
            node_ref=el
            on:mousemove=on_move
            style=("--mx", move || format!("{:.0}px", pos.get().0))
            style=("--my", move || format!("{:.0}px", pos.get().1))
        >
            {children()}
        </div>
    }
}

/// Infinite horizontal marquee ("Infinite Moving Cards"). Content is
/// rendered twice so the loop is seamless; pauses on hover.
#[component]
pub fn Marquee(children: ChildrenFn) -> impl IntoView {
    view! {
        <div class="marquee">
            <div class="marquee-track">
                <div class="marquee-group">{children()}</div>
                <div class="marquee-group" aria-hidden="true">{children()}</div>
            </div>
        </div>
    }
}

/// Area sparkline from raw values.
#[component]
pub fn Sparkline(
    #[prop(into)] values: Signal<Vec<f64>>,
    #[prop(default = 240.0)] width: f64,
    #[prop(default = 56.0)] height: f64,
    #[prop(default = "spark")] class: &'static str,
) -> impl IntoView {
    let path = move || {
        let v = values.get();
        if v.len() < 2 {
            return (String::new(), String::new());
        }
        let max = v.iter().cloned().fold(0.0, f64::max).max(1.0);
        let step = width / (v.len() - 1) as f64;
        let pts: Vec<(f64, f64)> =
            v.iter().enumerate().map(|(i, y)| (i as f64 * step, height - 4.0 - (y / max) * (height - 8.0))).collect();
        let line = pts.iter().enumerate().map(|(i, (x, y))| format!("{}{x:.1},{y:.1}", if i == 0 { "M" } else { "L" })).collect::<String>();
        let area = format!("{line}L{width:.1},{height:.1}L0,{height:.1}Z");
        (line, area)
    };
    view! {
        <svg class=class viewBox=format!("0 0 {width} {height}") preserveAspectRatio="none" aria-hidden="true">
            <defs>
                <linearGradient id="spark-fill" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stop-color="#a855f7" stop-opacity="0.45"></stop>
                    <stop offset="100%" stop-color="#a855f7" stop-opacity="0"></stop>
                </linearGradient>
            </defs>
            <path d=move || path().1 fill="url(#spark-fill)"></path>
            <path d=move || path().0 fill="none" stroke="url(#beam-grad-h)" stroke-width="2" vector-effect="non-scaling-stroke"></path>
            <defs>
                <linearGradient id="beam-grad-h" x1="0" y1="0" x2="1" y2="0">
                    <stop offset="0%" stop-color="#22d3ee"></stop>
                    <stop offset="100%" stop-color="#d946ef"></stop>
                </linearGradient>
            </defs>
        </svg>
    }
}

/// Bar chart with an optional second series stacked on top.
#[component]
pub fn Bars(
    #[prop(into)] values: Signal<Vec<(f64, f64)>>,
    #[prop(default = 600.0)] width: f64,
    #[prop(default = 160.0)] height: f64,
) -> impl IntoView {
    view! {
        <svg class="bars" viewBox=format!("0 0 {width} {height}") preserveAspectRatio="none" aria-hidden="true">
            {move || {
                let v = values.get();
                let n = v.len().max(1) as f64;
                let max = v.iter().map(|(a, b)| a + b).fold(0.0, f64::max).max(1.0);
                let bw = width / n;
                v.into_iter()
                    .enumerate()
                    .map(|(i, (a, b))| {
                        let ha = a / max * (height - 6.0);
                        let hb = b / max * (height - 6.0);
                        let x = i as f64 * bw + bw * 0.15;
                        let w = bw * 0.7;
                        view! {
                            <rect class="bar-a" x=x y=height - ha width=w height=ha rx="2"></rect>
                            <rect class="bar-b" x=x y=height - ha - hb width=w height=hb rx="2"></rect>
                        }
                    })
                    .collect_view()
            }}
        </svg>
    }
}
