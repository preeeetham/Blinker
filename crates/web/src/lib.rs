//! Blinker's UI, written in Leptos. Server-rendered by the Axum backend and
//! hydrated in the browser as WebAssembly.

pub mod api;
pub mod app;
pub mod fx;
pub mod pages;
pub mod util;
pub mod wallet;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
