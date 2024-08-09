use std::{convert::identity, time::Duration};

use leptos::{html::Div, prelude::*};
use tailwind_merge::tw;
use web_sys::HtmlDivElement;

use crate::{
    components::dice::Dice,
    game_state::{Cell, GameState, Player, CELLS_COUNT},
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
    let is_property = move || matches!(current_cell(), Cell::Property(_));
    let bg_color = move || {
        if let Cell::Property(prop) = current_cell() {
            prop.data.group.color
        } else {
            "#fff"
        }
    };

    view! {
        <div
            node_ref=node_ref
            class=move || tw!("p-1", !is_property() => "text-black")
            style:background-color=bg_color
        >
            {match current_cell() {
                Cell::Start => "Start".to_owned(),
                Cell::Jail => "Jail".to_owned(),
                Cell::FreeParking => "FreeParking".to_owned(),
                Cell::GoToJail => "GoToJail".to_owned(),
                Cell::Tax(tax) => format!("Tax: {}", tax),
                Cell::Chance => "Chance".to_owned(),
                Cell::Property(prop) => format!("Property: {}", prop.data.title),
            }}

        </div>
    }
}

#[component]
pub fn PlayerCard(player: Player, #[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    let game_state = GameState::use_context();

    let is_current_player = move || game_state.current_player() == player;
    let bg = move || {
        is_current_player()
            .then(|| player.color.get_player_gradient())
            .unwrap_or_default()
    };

    view! {
        <div
            style:background=bg
            class=move || {
                tw!(
                    "flex flex-col items-center justify-center p-3 bg-gray-900 transition-all",
                    is_current_player() => "scale-[1.075]",
                    class()
                )
            }
        >
            <div
                class="w-14 h-14 bg-gray-900 rounded-full"
                style:background=move || {
                    (!is_current_player())
                        .then(|| player.color.get_player_gradient())
                        .unwrap_or_default()
                }
            />
            <div class="mt-2 text-sm font-bold">{player.id}": "{player.name.get_value()}</div>
            <div class="mt-4 text-2xl">
                <span class="inline-block pr-0.5 opacity-50 scale-75">"$"</span>
                {move || player.balance().to_string()}
                <span class="pl-0.5 opacity-70">"k"</span>
            </div>
        </div>
    }
}

#[component]
fn PlayerToken(cells_refs: CellsRefs, player: Player) -> impl IntoView {
    let width = 32f64;
    let height = 32f64;
    let gap = 6f64;

    let game_state = GameState::use_context();
    let (scroll_x, scroll_y) = use_window_scroll();
    let coordinates = RwSignal::new(None);

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

        coordinates.set(Some((x, y)));
    });

    view! {
        <div
            on:transitionend=move |_| game_state.player_token_transition_end()
            style:left=move || coordinates().map(|(x, _)| format!("{x}px")).unwrap_or_default()
            style:top=move || coordinates().map(|(_, y)| format!("{y}px")).unwrap_or_default()
            style:width=format!("{width}px")
            style:height=format!("{height}px")
            style:background=player.color.get_cell_gradient()
            class=move || {
                tw!(
                    "absolute left-0 top-0 rounded-full [border:3px_solid_rgba(0,0,0,.5)] shadow-[0_0_15px_rgba(0,0,0,.667)] transition-all duration-1000",
                    coordinates().is_none() => "hidden"
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
