use leptos::prelude::*;
use leptos_router::components::Router;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_wasm::WASMLayer;

use self::{app::App, utils::fmt_panic};

mod app;
mod components;
mod game_data;
mod game_state;
mod hooks;
mod pages;
mod utils;

fn main() {
    setup_error_handlers();

    mount_to_body(|| {
        view! {
            <Router>
                <App />
            </Router>
        }
    });
}

fn setup_error_handlers() {
    std::panic::set_hook(Box::new(fmt_panic::panic_hook));
    tracing_subscriber::registry()
        .with(WASMLayer::default())
        .with(ErrorLayer::default())
        .init();
}
