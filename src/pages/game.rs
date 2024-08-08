use std::{convert::identity, time::Duration};

use leptos::{html::Div, prelude::*};
use tailwind_merge::tw;
use web_sys::HtmlDivElement;

use crate::{
    components::dice::Dice,
    game_state::{CellType, GameState, Player, CELLS_COUNT},
    hooks::window_scroll::use_window_scroll,
};

#[derive(Debug, Clone, Copy)]
struct CellsRefs(StoredValue<[NodeRef<Div>; CELLS_COUNT]>);

impl CellsRefs {
    fn new() -> Self {
        Self(StoredValue::new([(); CELLS_COUNT].map(|_| NodeRef::new())))
    }

    fn set(&self, index: usize) -> NodeRef<Div> {
        assert!(
            index < CELLS_COUNT,
            "There is only {CELLS_COUNT} cells, dummy. Provided index: {index}"
        );
        self.0.with_value(|refs| refs[index])
    }

    fn get(&self, index: usize) -> HtmlDivElement {
        self.0.get_value()[index]
            .get()
            .unwrap_or_else(|| panic!("Reference for cell {index} should be initialized"))
    }
}

#[component]
pub fn GamePage() -> impl IntoView {
    let cells_refs = CellsRefs::new();
    let game_state = GameState::new();
    provide_context(game_state);

    view! {
        <div class="grid gap-2 p-3 min-h-full grid-cols-[200px_auto] grid-rows-[repeat(5,1fr)] grow">
            {move || {
                game_state
                    .get_players()
                    .into_values()
                    .map(|player| view! { <PlayerCard player class="col-[1]".to_owned() /> })
                    .collect_view()
            }}
            <div class="grid relative gap-0.5 min-h-full grid-columns-[2fr_repeat(9,21fr)_2fr] grid-rows-[2fr_repeat(9,1fr)_2fr] col-[2] row-[1/6]">
                <Rows cells_refs />
                <Chat class="col-[2/11] row-[2/11]".to_owned() />
                {move || {
                    game_state
                        .render_dice()
                        .map(|(a, b)| {
                            view! {
                                <div class="flex absolute top-1/2 left-1/2 gap-4 -translate-x-1/2 -translate-y-1/2">
                                    <Dice
                                        side=a
                                        animated=true
                                        on_animation_end=Callback::new(move |_| {
                                            set_timeout(
                                                move || game_state.dice_transition_end(),
                                                Duration::from_millis(200),
                                            );
                                        })
                                    />
                                    <Dice side=b animated=true />
                                </div>
                            }
                        })
                }}
            </div>
            {move || {
                game_state
                    .get_players()
                    .into_values()
                    .map(|player| view! { <PlayerToken cells_refs player /> })
                    .collect_view()
            }}
        </div>
    }
}

#[component]
fn Rows(cells_refs: CellsRefs) -> impl IntoView {
    (0..40)
        .map(|i| {
            let column = move || match i {
                0..10 => i + 1,
                10..20 => 11,
                20..30 => 31 - i,
                30..40 => 1,
                _ => unreachable!("There is only {CELLS_COUNT} cells, silly. ᓚ₍ ^. .^₎"),
            };

            let row = move || match i {
                0..10 => 1,
                10..20 => i - 9,
                20..30 => 11,
                30..40 => 41 - i,
                _ => unreachable!("There is only {CELLS_COUNT} cells, silly. ᓚ₍ ^. .^₎"),
            };

            view! {
                <Cell
                    node_ref=cells_refs.set(i)
                    // I need to wrap `i` in something, or else `{..}` is incorrectly parsed as
                    // an `i{..}` function call, which breaks "smart spread" in leptos.
                    // I cannot use `{i]`, because leptosfmt removes braces...
                    // TODO: Make PR into leptosfmt to add `#[leptosfmt::skip]`
                    index=identity(i)
                    // This does not work because of `!contains_dash` in leptos_macro/src/view/mod.rs:482.
                    // TODO: I should make issue/pr in leptos.
                    // style:grid-column=column().to_string()
                    {..}
                    style=("grid-column", column().to_string())
                    style=("grid-row", row().to_string())
                />
            }
        })
        .collect_view()
}

#[component]
fn Cell(index: usize, node_ref: NodeRef<Div>) -> impl IntoView {
    let game_state = GameState::use_context();
    let current_cell = move || game_state.get_cell(index);
    let bg_class = move || match current_cell().ty {
        CellType::Start => "bg-green-500",
        CellType::Jail => "bg-yellow-500",
        CellType::FreeParking => "bg-red-500",
        CellType::GoToJail => "bg-purple-500",
        CellType::Property => "bg-gray-500",
    };

    view! {
        <div node_ref=node_ref class=move || tw!("p-1", bg_class())>
            {match current_cell().ty {
                CellType::Start => "Start",
                CellType::Jail => "Jail",
                CellType::FreeParking => "FreeParking",
                CellType::GoToJail => "GoToJail",
                CellType::Property => "Property",
            }}

        </div>
    }
}

#[component]
pub fn PlayerCard(player: Player, #[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    let game_state = GameState::use_context();

    let background_color = move || {
        (game_state.current_player() == player)
            .then(|| player.color.get_value())
            .unwrap_or_default()
    };

    view! {
        <div style:background-color=background_color class=move || tw!("bg-gray-500", class())>
            <div>"Id: " {player.id}</div>
            <div>"Name: " {player.name.get_value()}</div>
            <div style:color=player.color.get_value()>"Color"</div>
        </div>
    }
}

#[component]
fn PlayerToken(cells_refs: CellsRefs, player: Player) -> impl IntoView {
    let width = 12f64;
    let height = 12f64;
    let gap = 6f64;

    let game_state = GameState::use_context();
    let (scroll_x, scroll_y) = use_window_scroll();
    let transform = RwSignal::new(String::new());

    Effect::new(move |_| {
        let players = game_state.get_players_by_cell(player.position());
        let current_player_index = players
            .iter()
            .position(|p| p == &player)
            .expect("game_state.get_players_by_cell() should contain current player also");
        let current_cell = cells_refs.get(player.position());
        let cell_rect = current_cell.get_bounding_client_rect();

        let cell_x_center = scroll_x() + cell_rect.x() + cell_rect.width() / 2f64;
        let cell_y_center = scroll_y() + cell_rect.y() + cell_rect.height() / 2f64;

        let token_box_height = height * players.len() as f64 + gap * (players.len() - 1) as f64;

        let x = cell_x_center - width / 2f64;
        let y = cell_y_center - token_box_height / 2f64
            + height * current_player_index as f64
            + gap * current_player_index as f64;

        transform.set(format!("translate({x}px, {y}px)"));
    });

    view! {
        <div
            on:transitionend=move |_| game_state.player_token_transition_end()
            style:transform=transform
            style:width=width.to_string()
            style:height=height.to_string()
            style:background-color=player.color.get_value()
            class=move || {
                tw!(
                    "absolute left-0 top-0 w-3 h-3 border-gray-600 border transition-transform",
                    transform().is_empty() => "hidden"
                )
            }
        />
    }
}

#[component]
pub fn Chat(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    let game_state = GameState::use_context();
    let roll_dice_action = Action::new_local(move |()| async move { game_state.roll_dice().await });
    let roll_dice_pending = roll_dice_action.pending();

    let roll_dice = move |_| {
        if roll_dice_pending() {
            return;
        };

        roll_dice_action.dispatch(());
    };

    view! {
        <div
            class=move || tw!("bg-cyan-700", !roll_dice_pending() => "cursor-pointer", class())
            on:click=roll_dice
        >
            "Chat"
        </div>
    }
}
