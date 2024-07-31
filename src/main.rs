use leptos::prelude::*;
use std::panic;
use tailwind_merge::tw;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_wasm::WASMLayer;

mod stack_trace;

fn main() {
    setup_error_handlers();

    mount_to_body(|| view! { <p class=tw!("text-red-600")>"Hello, world!"</p> })
}

fn setup_error_handlers() {
    tracing_subscriber::registry()
        .with(WASMLayer::default())
        .init();

    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default().into_hooks();
    eyre_hook.install().unwrap();

    panic::set_hook(Box::new(move |info: &panic::PanicHookInfo| {
        tracing::error!(
            "{}",
            stack_trace::with_stack_trace(panic_hook.panic_report(info).to_string())
        );
    }));
}
