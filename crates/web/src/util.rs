use leptos::prelude::*;

pub fn short(addr: &str) -> String {
    if addr.len() <= 10 { addr.to_string() } else { format!("{}…{}", &addr[..4], &addr[addr.len() - 4..]) }
}

pub fn fmt_num(v: f64) -> String {
    if v >= 1e9 {
        format!("{:.1}B", v / 1e9)
    } else if v >= 1e6 {
        format!("{:.1}M", v / 1e6)
    } else if v >= 1e4 {
        format!("{:.1}K", v / 1e3)
    } else if v.fract() == 0.0 {
        format!("{v:.0}")
    } else if v >= 100.0 {
        format!("{v:.1}")
    } else {
        let s = format!("{v:.4}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

pub fn fmt_bytes(b: f64) -> String {
    const U: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut v = b;
    let mut i = 0;
    while v >= 1024.0 && i < U.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 { format!("{v:.0} {}", U[i]) } else { format!("{v:.1} {}", U[i]) }
}

pub fn fmt_us(us: f64) -> String {
    if us >= 1e6 {
        format!("{:.2}s", us / 1e6)
    } else if us >= 1e3 {
        format!("{:.1}ms", us / 1e3)
    } else {
        format!("{us:.0}µs")
    }
}

pub fn copy_to_clipboard(text: &str) {
    if let Some(w) = web_sys::window() {
        let _ = w.navigator().clipboard().write_text(text);
    }
}

pub fn open_url(url: &str) {
    if let Some(w) = web_sys::window() {
        let _ = w.open_with_url_and_target(url, "_blank");
    }
}

// ───────────────────────────── toasts ─────────────────────────────

#[derive(Clone, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub ok: bool,
    pub text: String,
}

#[derive(Clone, Copy)]
pub struct Toasts(pub RwSignal<Vec<Toast>>);

pub fn toast(ok: bool, text: impl Into<String>) {
    if let Some(Toasts(list)) = use_context::<Toasts>() {
        let id = js_sys::Date::now() as u64 ^ (js_sys::Math::random() * 1e9) as u64;
        list.update(|l| l.push(Toast { id, ok, text: text.into() }));
        set_timeout(move || list.update(|l| l.retain(|t| t.id != id)), std::time::Duration::from_millis(4200));
    }
}

#[component]
pub fn ToastHost() -> impl IntoView {
    let Toasts(list) = expect_context::<Toasts>();
    view! {
        <div class="toasts" aria-live="polite">
            {move || {
                list.get()
                    .into_iter()
                    .map(|t| {
                        view! {
                            <div class=if t.ok { "toast ok" } else { "toast err" }>
                                <span class="toast-dot"></span>
                                {t.text}
                            </div>
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}

/// Runs `f` now and then every `ms` on the client; stops when the owner is
/// disposed (navigating away). No-op during SSR.
pub fn poll(ms: u64, f: impl Fn() + Clone + 'static) {
    Effect::new(move |_| {
        f();
        if let Ok(handle) = set_interval_with_handle(f.clone(), std::time::Duration::from_millis(ms)) {
            on_cleanup(move || handle.clear());
        }
    });
}
