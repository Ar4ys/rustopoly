use leptos::prelude::*;
use leptos_router::{
    components::{Route, Routes},
    StaticSegment,
};

use crate::{
    hooks::redirect::use_redirect,
    pages::{game::GamePage, login::LoginPage},
};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Routes fallback=|| use_redirect("/", Default::default())>
            <Route path=StaticSegment("") view=LoginPage/>
            <Route path=StaticSegment("game") view=GamePage/>
        </Routes>
    }
}
