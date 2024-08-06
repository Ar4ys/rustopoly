use leptos::prelude::*;
use std::panic;
use tailwind_merge::tw;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_wasm::WASMLayer;

mod fmt_panic;

#[tracing::instrument]
fn oops(arg: u32, arg2: u32) {
    panic!("Oh well ._.");
}

fn main() {
    setup_error_handlers();
    oops(12, 21);

    mount_to_body(|| view! { <p class=tw!("text-red-600")>"Hello, world!"</p> })
}

fn setup_error_handlers() {
    tracing_subscriber::registry()
        .with(WASMLayer::default())
        .with(ErrorLayer::default())
        .init();

    panic::set_hook(Box::new(fmt_panic::panic_hook));
}
