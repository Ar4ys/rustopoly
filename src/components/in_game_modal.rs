use futures::{
    channel::{mpsc, oneshot},
    FutureExt, SinkExt, TryFutureExt,
};
use leptos::{
    either::{Either, EitherOf3},
    prelude::*,
    spawn::spawn_local,
};
use tailwind_merge::tw;

use crate::game_state::GameState;

#[component]
pub fn InGameModal() -> impl IntoView {
    let game_state = GameState::use_context();
    let is_hidden = move || matches!(game_state.in_game_modal_state.get(), InGameModalState::None);

    view! {
        <div class=move || {
            tw!(
                "absolute top-2 right-2 left-2 p-3 text-black bg-white rounded-md", is_hidden() => "hidden"
            )
        }>
            {move || match game_state.in_game_modal_state.get() {
                InGameModalState::None => EitherOf3::A(()),
                InGameModalState::OneButton { text, button_text, channel } => {
                    EitherOf3::B(
                        view! {
                            <>
                                <p>{text}</p>
                                <button
                                    class="p-2 mt-3 rounded border-2"
                                    on:click={
                                        let channel = channel.clone();
                                        move |ev| {
                                            ev.stop_propagation();
                                            let mut channel = channel.clone();
                                            spawn_local(async move {
                                                let _ = channel.send(()).await;
                                            });
                                        }
                                    }
                                >
                                    {button_text}
                                </button>
                            </>
                        },
                    )
                }
                InGameModalState::TwoButton {
                    text,
                    ok_button_text,
                    cancel_button_text,
                    channel,
                } => {
                    EitherOf3::C(
                        view! {
                            <>
                                <p>{text}</p>
                                // TODO: Rework this stupidity. Maybe channels wasn't a great idea.
                                <button
                                    class="p-2 mt-3 rounded border-2"
                                    on:click={
                                        let channel = channel.clone();
                                        move |ev| {
                                            ev.stop_propagation();
                                            let mut channel = channel.clone();
                                            spawn_local(async move {
                                                let _ = channel.send(ModalResponse::Ok).await;
                                            });
                                        }
                                    }
                                >
                                    {ok_button_text}
                                </button>
                                <button
                                    class="p-2 ml-3 rounded border-2"
                                    on:click={
                                        let channel = channel.clone();
                                        move |ev| {
                                            ev.stop_propagation();
                                            let mut channel = channel.clone();
                                            spawn_local(async move {
                                                let _ = channel.send(ModalResponse::Cancel).await;
                                            });
                                        }
                                    }
                                >
                                    {cancel_button_text}
                                </button>
                            </>
                        },
                    )
                }
            }}
        </div>
    }
}

#[derive(Debug, Default, Clone)]
pub enum InGameModalState {
    #[default]
    None,
    OneButton {
        text: String,
        button_text: String,
        channel: mpsc::UnboundedSender<()>,
    },
    TwoButton {
        text: String,
        ok_button_text: String,
        cancel_button_text: String,
        channel: mpsc::UnboundedSender<ModalResponse>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ModalResponse {
    Ok,
    Cancel,
}

impl InGameModalState {
    pub fn new() -> Self {
        Self::None
    }
}
