use std::{convert::identity, ops::Deref, time::Duration};

use leptos::{
    either::{Either, EitherOf3},
    ev,
    html::Div,
    prelude::*,
    spawn::spawn_local,
};
use tailwind_merge::tw;
use web_sys::{HtmlDivElement, HtmlElement, Node};

use crate::{
    cell::{Cell, Property, PropertyType, CELLS_COUNT},
    components::{dice::Dice, in_game_modal::InGameModal},
    game_state::GameState,
    hooks::window_scroll::use_window_scroll,
    player::Player,
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
    let is_dice_shown = RwSignal::new(false);
    let cells_refs = CellsRefs::new();
    let game_state = GameState::new();
    provide_context(game_state);

    Effect::new(move |_| is_dice_shown.set(game_state.rolled_dice().is_some()));

    view! {
        <div class="grid gap-9 p-7 h-screen grid-cols-[200px_auto] grid-rows-[repeat(5,1fr)] grow">
            {move || {
                game_state
                    .get_players()
                    .into_values()
                    .map(|player| view! { <PlayerCard player class="col-[1]".to_owned() /> })
                    .collect_view()
            }}
            <div class="grid relative gap-0.5 min-h-full max-w-[50vw] grid-columns-[2fr_repeat(9,21fr)_2fr] grid-rows-[2fr_repeat(9,1fr)_2fr] col-[2] row-[1/6]">
                <Rows cells_refs />
                <Chat class="col-[2/11] row-[2/11]".to_owned() />
                {move || {
                    game_state
                        .rolled_dice()
                        .filter(|_| is_dice_shown.get())
                        .map(|(a, b)| {
                            view! {
                                <div class="flex absolute top-1/2 left-1/2 gap-4 -translate-x-1/2 -translate-y-1/2">
                                    <Dice
                                        side=a
                                        animated=true
                                        on_animation_end=Callback::new(move |_| {
                                            set_timeout(
                                                move || {
                                                    is_dice_shown.set(false);
                                                    game_state.dice_transition_end();
                                                },
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
    let is_info_open = RwSignal::new(false);
    let current_cell = move || game_state.get_cell(index);
    let cell_bg = move || {
        current_cell()
            .try_unwrap_property()
            .ok()
            .and_then(|prop| prop.owner.get())
            .map(|owner| owner.color.get_cell_gradient())
            .unwrap_or("#fff")
    };
    let rent_bg = move || {
        current_cell()
            .try_unwrap_property()
            .map(|prop| prop.data.group.color)
            .unwrap_or_default()
    };

    let handle_click = move |_| {
        if let Cell::Property(_) = current_cell() {
            is_info_open.update(|is_info_open| *is_info_open = !*is_info_open)
        }
    };

    let handle = window_event_listener(ev::click, move |event| {
        let Some(node_ref) = node_ref.get_untracked() else {
            return;
        };

        // TODO: When you press on cell - it automatically closes

        let is_click_inside_cell = node_ref.contains(Some(&event_target::<Node>(&event)));

        if !is_click_inside_cell {
            is_info_open.set(false)
        }
    });

    on_cleanup(move || handle.remove());

    // let trigger_on_click =
    //     move |_| spawn_local(async move { untrack(current_cell).trigger(&game_state).await });

    view! {
        <div
            node_ref=node_ref
            class=move || tw!("relative p-1 cursor-pointer", cell_bg() == "#fff" => "text-black")
            style:background=cell_bg
            on:click=handle_click
        >
            {match current_cell() {
                Cell::Start => "Start".into_any(),
                Cell::Jail => "Jail".into_any(),
                Cell::FreeParking => "FreeParking".into_any(),
                Cell::GoToJail => "GoToJail".into_any(),
                Cell::Tax(tax) => format!("Tax: {}", tax).into_any(),
                Cell::Chance => "Chance".into_any(),
                Cell::Property(prop) => {
                    view! {
                        <>
                            <Show when=is_info_open>
                                <PropertyInfo
                                    property=prop
                                    class=tw!(
                                        "absolute z-10",
                                        match index {
                                            0..10 => "left-1/2 top-full -translate-x-1/2 translate-y-2",
                                            10..20 => "top-1/2 right-full -translate-x-2 -translate-y-1/2",
                                            20..30 => "left-1/2 bottom-full -translate-x-1/2 -translate-y-2",
                                            30..40 => "top-1/2 left-full translate-x-2 -translate-y-1/2",
                                            _ => unreachable!(),
                                        }
                                    )
                                />
                            </Show>
                            {format!("Property: {}", prop.data.title)}
                            <div
                                style:background=rent_bg
                                class=tw!(
                                    "absolute leading-7 text-center text-white",
                                    match index {
                                        0..10 => "left-0 -top-7 w-full h-7",
                                        10..20 => "top-0 -right-7 h-full w-7 [writing-mode:vertical-lr]",
                                        20..30 => "left-0 -bottom-7 w-full h-7",
                                        30..40 => "top-0 -left-7 h-full w-7 [writing-mode:vertical-lr] rotate-180",
                                        _ => unreachable!(),
                                    }
                                )
                            >
                                {move || {
                                    if let (PropertyType::Utility { levels }, Some(owner)) = (
                                        prop.ty,
                                        prop.owner.get(),
                                    ) {
                                        let (owns, _) = game_state
                                            .has_from_group(&owner, &prop.data.group);
                                        format!("x{}", levels[owns - 1])
                                    } else {
                                        format!(
                                            "{}k",
                                            prop.rent(&game_state).unwrap_or(prop.data.price),
                                        )
                                    }
                                }}
                            </div>
                        </>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

#[component]
pub fn PropertyInfo(
    #[prop(optional)] node_ref: NodeRef<Div>,
    property: Property,
    class: String,
) -> impl IntoView {
    view! {
        <div node_ref=node_ref class=tw!("rounded-md overflow-hidden w-52", class)>
            <div class="py-3 px-4 text-white" style:background=property.data.group.color>
                <div class="text-lg font-bold">{property.data.title}</div>
                <div class="-mt-0.5 text-xs font-bold opacity-75">{property.data.group.title}</div>
            </div>
            <div class="flex flex-col gap-2.5 py-3 px-4 text-gray-500 bg-white">
                <div class="leading-[14px]">
                    {move || match property.ty {
                        PropertyType::Simple { .. } => "Build agencies to make rent higher.",
                        PropertyType::Transport { .. } => {
                            "Rent depends on amount of autos you have."
                        }
                        PropertyType::Utility { .. } => {
                            "Rent depends on the dice sum and amount of developers you have."
                        }
                    }}
                </div>
                <div>
                    {move || match property.ty {
                        PropertyType::Simple { levels, .. } => {
                            EitherOf3::A(
                                view! {
                                    <>
                                        <div class="flex justify-between">
                                            <span>"Base rent"</span>
                                            <span>
                                                {move || levels[0].to_string()}
                                                <span class="opacity-70">"k"</span>
                                            </span>
                                        </div>
                                        <div class="flex justify-between">
                                            <span>"★"</span>
                                            <span>
                                                {move || levels[1].to_string()}
                                                <span class="opacity-70">"k"</span>
                                            </span>
                                        </div>
                                        <div class="flex justify-between">
                                            <span>"★ ★"</span>
                                            <span>
                                                {move || levels[2].to_string()}
                                                <span class="opacity-70">"k"</span>
                                            </span>
                                        </div>
                                        <div class="flex justify-between">
                                            <span>"★ ★ ★"</span>
                                            <span>
                                                {move || levels[3].to_string()}
                                                <span class="opacity-70">"k"</span>
                                            </span>
                                        </div>
                                        <div class="flex justify-between">
                                            <span>"★ ★ ★ ★"</span>
                                            <span>
                                                {move || levels[4].to_string()}
                                                <span class="opacity-70">"k"</span>
                                            </span>
                                        </div>
                                        <div class="flex justify-between">
                                            <span class="text-yellow-500 scale-150">"★"</span>
                                            <span>
                                                {move || levels[5].to_string()}
                                                <span class="opacity-70">"k"</span>
                                            </span>
                                        </div>
                                    </>
                                },
                            )
                        }
                        PropertyType::Transport { levels } => {
                            EitherOf3::B(
                                view! {
                                    <>
                                        <div class="flex justify-between">
                                            <span>"1 field"</span>
                                            <span>
                                                {move || levels[0].to_string()}
                                                <span class="opacity-70">"k"</span>
                                            </span>
                                        </div>
                                        <div class="flex justify-between">
                                            <span>"2 fields"</span>
                                            <span>
                                                {move || levels[1].to_string()}
                                                <span class="opacity-70">"k"</span>
                                            </span>
                                        </div>
                                        <div class="flex justify-between">
                                            <span>"3 fields"</span>
                                            <span>
                                                {move || levels[2].to_string()}
                                                <span class="opacity-70">"k"</span>
                                            </span>
                                        </div>
                                        <div class="flex justify-between">
                                            <span>"4 fields"</span>
                                            <span>
                                                {move || levels[3].to_string()}
                                                <span class="opacity-70">"k"</span>
                                            </span>
                                        </div>
                                    </>
                                },
                            )
                        }
                        PropertyType::Utility { levels } => {
                            EitherOf3::C(
                                view! {
                                    <>
                                        <div class="flex justify-between">
                                            <span>"1 field"</span>
                                            <span>
                                                <span class="opacity-70">"x"</span>
                                                {move || levels[0].to_string()}
                                            </span>
                                        </div>
                                        <div class="flex justify-between">
                                            <span>"2 fields"</span>
                                            <span>
                                                <span class="opacity-70">"x"</span>
                                                {move || levels[1].to_string()}
                                            </span>
                                        </div>
                                    </>
                                },
                            )
                        }
                    }}
                </div>
                <div>
                    <div class="flex justify-between">
                        <span>"Field's price"</span>
                        <span>
                            {move || property.data.price.to_string()}
                            <span class="pl-0.5 opacity-70">"k"</span>
                        </span>
                    </div>
                    <div class="flex justify-between">
                        <span>"Field's mortgage"</span>
                        <span>
                            {move || property.reward_for_mortgaging().to_string()}
                            <span class="pl-0.5 opacity-70">"k"</span>
                        </span>
                    </div>
                    <div class="flex justify-between">
                        <span>"Field's recovery"</span>
                        <span>
                            {move || property.recovery_price().to_string()}
                            <span class="pl-0.5 opacity-70">"k"</span>
                        </span>
                    </div>
                    {move || {
                        if let PropertyType::Simple { level_price, .. } = property.ty {
                            Either::Left(
                                view! {
                                    <div class="flex justify-between">
                                        <span>"Agency price"</span>
                                        <span>
                                            {move || level_price.to_string()}
                                            <span class="pl-0.5 opacity-70">"k"</span>
                                        </span>
                                    </div>
                                },
                            )
                        } else {
                            Either::Right(())
                        }
                    }}
                </div>
            </div>
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
                    is_current_player() => "scale-105",
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
            class=move || {
                tw!("relative bg-cyan-700", !roll_dice_pending() => "cursor-pointer", class())
            }
            on:click=roll_dice
        >
            "Chat"
            <InGameModal />
        </div>
    }
}
