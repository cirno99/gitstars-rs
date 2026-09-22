#![recursion_limit = "512"]

pub mod app;
pub mod components;
pub mod constants;
pub mod i18n;
pub mod loaders;
pub mod models;
pub mod server_fn;
pub mod state;

#[cfg(feature = "ssr")]
pub mod api;
#[cfg(feature = "ssr")]
pub mod auth;
#[cfg(feature = "ssr")]
pub mod cache;
#[cfg(feature = "ssr")]
pub mod config;
#[cfg(feature = "ssr")]
pub mod ranking;
#[cfg(feature = "ssr")]
pub mod readme;
#[cfg(feature = "ssr")]
pub mod stars;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
