use std::sync::Arc;

use derive_more::derive::Debug;
use futures::{channel::mpsc, SinkExt, StreamExt};
use leptos::{either::EitherOf3, prelude::*, spawn::spawn_local};
use tailwind_merge::tw;

use crate::game_state::GameState;

#[component]
pub fn InGameModal() -> impl IntoView {
    let game_state = GameState::use_context();
    let is_hidden = move || game_state.in_game_modal_state.ty.get().is_none();

    Effect::new(move |_| {
        if game_state.self_player.has_lost() {
            game_state.in_game_modal_state.ty.set(None);
        }
    });

    view! {
        <div class=move || {
            tw!(
                "absolute top-2 right-2 left-2 p-3 text-black bg-white rounded-md",
                is_hidden() => "hidden"
            )
        }>
            {move || match game_state.in_game_modal_state.ty.get() {
                None => EitherOf3::A(()),
                Some(InGameModalStateType::OneButton { text, button_text, on_click }) => {
                    EitherOf3::B(
                        view! {
                            <InGameModalView
                                text
                                left_button_text=button_text
                                on_left_button_click=Callback::new(move |_| on_click())
                            />
                        },
                    )
                }
                Some(
                    InGameModalStateType::TwoButtons {
                        text,
                        ok_button_text,
                        cancel_button_text,
                        on_ok_click,
                        on_cancel_click,
                    },
                ) => {
                    EitherOf3::C(
                        view! {
                            <InGameModalView
                                text
                                left_button_text=ok_button_text
                                on_left_button_click=Callback::new(move |_| on_ok_click())
                                right_button_text=cancel_button_text
                                on_right_button_click=Callback::new(move |_| on_cancel_click())
                            />
                        },
                    )
                }
            }}
        </div>
    }
}

#[component]
pub fn InGameModalView(
    text: String,
    left_button_text: String,
    #[prop(into)] on_left_button_click: Callback<()>,
    #[prop(optional)] right_button_text: String,
    #[prop(optional, into)] on_right_button_click: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <>
            <p>{text}</p>
            <button class="p-2 mt-3 rounded border-2" on:click=move |_| on_left_button_click(())>
                {left_button_text}
            </button>
            {move || {
                on_right_button_click
                    .map(|on_right_button_click| {
                        view! {
                            <button
                                class="p-2 ml-3 rounded border-2"
                                on:click=move |_| on_right_button_click(())
                            >
                                {right_button_text.clone()}
                            </button>
                        }
                    })
            }}
        </>
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct InGameModalState {
    ty: RwSignal<Option<InGameModalStateType>>,
}

#[derive(Debug, Clone)]
pub enum InGameModalStateType {
    OneButton {
        text: String,
        button_text: String,
        #[debug(skip)]
        on_click: Arc<dyn Fn() + Send + Sync>,
    },
    TwoButtons {
        text: String,
        ok_button_text: String,
        cancel_button_text: String,
        #[debug(skip)]
        on_ok_click: Arc<dyn Fn() + Send + Sync>,
        #[debug(skip)]
        on_cancel_click: Arc<dyn Fn() + Send + Sync>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ModalResponse {
    Ok,
    Cancel,
}

impl InGameModalState {
    pub fn new() -> Self {
        Self {
            ty: RwSignal::new(None),
        }
    }

    pub fn one_button(
        &self,
        text: &str,
        button_text: &str,
        on_click: impl Fn() + Send + Sync + 'static,
    ) {
        let this = *self;
        self.ty.set(Some(InGameModalStateType::OneButton {
            text: text.to_owned(),
            button_text: button_text.to_owned(),
            on_click: Arc::new(move || {
                this.ty.set(None);
                on_click();
            }),
        }))
    }

    pub fn two_buttons(
        &self,
        text: &str,
        ok_button_text: &str,
        cancel_button_text: &str,
        on_ok_click: impl Fn() + Send + Sync + 'static,
        on_cancel_click: impl Fn() + Send + Sync + 'static,
    ) {
        let this = *self;
        self.ty.set(Some(InGameModalStateType::TwoButtons {
            text: text.to_owned(),
            ok_button_text: ok_button_text.to_owned(),
            cancel_button_text: cancel_button_text.to_owned(),
            on_ok_click: Arc::new(move || {
                this.ty.set(None);
                on_ok_click();
            }),
            on_cancel_click: Arc::new(move || {
                this.ty.set(None);
                on_cancel_click();
            }),
        }));
    }

    pub async fn one_button_async(&self, text: &str, button_text: &str) {
        let (sender, mut receiver) = mpsc::unbounded();
        self.one_button(text, button_text, move || {
            let mut sender = sender.clone();
            spawn_local(async move {
                sender.send(()).await.unwrap();
            });
        });
        receiver.next().await;
    }

    pub async fn two_buttons_async(
        &self,
        text: &str,
        ok_button_text: &str,
        cancel_button_text: &str,
    ) -> ModalResponse {
        let (sender, mut receiver) = mpsc::unbounded();
        self.two_buttons(
            text,
            ok_button_text,
            cancel_button_text,
            {
                let sender = sender.clone();
                move || {
                    let mut sender = sender.clone();
                    spawn_local(async move {
                        sender.send(ModalResponse::Ok).await.unwrap();
                    });
                }
            },
            move || {
                let mut sender = sender.clone();
                spawn_local(async move {
                    sender.send(ModalResponse::Cancel).await.unwrap();
                });
            },
        );

        receiver
            .next()
            .await
            .expect("Receiver should not be closed")
    }
}
