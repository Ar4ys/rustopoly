use leptos::{ev, prelude::*};

pub fn use_window_scroll() -> (Signal<f64>, Signal<f64>) {
    let scroll_x = RwSignal::new(window().scroll_x().unwrap());
    let scroll_y = RwSignal::new(window().scroll_y().unwrap());

    let handler = window_event_listener(ev::resize, move |_| {
        scroll_x.set(window().scroll_x().unwrap());
        scroll_y.set(window().scroll_y().unwrap());
    });

    on_cleanup(move || handler.remove());

    (scroll_x.into(), scroll_y.into())
}
