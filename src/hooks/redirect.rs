use leptos::prelude::*;
use leptos_router::{hooks::use_navigate, NavigateOptions};

pub fn use_redirect(path: &str, options: NavigateOptions) {
    let navigate = use_navigate();
    let path = path.to_owned();
    request_animation_frame(move || navigate(&path, options));
}
