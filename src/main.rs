use leptos::prelude::*;
use leptos_router::components::Router;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_wasm::WASMLayer;
use wasm_bindgen::prelude::*;

use self::{app::App, utils::fmt_panic};

mod app;
mod cell;
mod components;
mod game_data;
mod game_state;
mod hooks;
mod pages;
mod player;
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
    window().set_onerror(Some(
        &Closure::<dyn Fn(_, _, _, _, _) -> _>::new(fmt_panic::uncaught_error_hook)
            .into_js_value()
            .unchecked_into(),
    ));
    tracing_subscriber::registry()
        .with(WASMLayer::default())
        .with(ErrorLayer::default())
        .init();
}
