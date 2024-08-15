use std::{convert::identity, time::Duration};

use leptos::{
    either::{Either, EitherOf3},
    ev,
    html::Div,
    portal::Portal,
    prelude::*,
};
use tailwind_merge::tw;
use web_sys::{HtmlDivElement, Node};

use crate::{
    cell::{
        BuildAgencyError, Cell, OwnedPropertyError, Property, PropertyType, SellAgencyError,
        CELLS_COUNT,
    },
    components::{dice::Dice, in_game_modal::InGameModal},
    game_state::GameState,
    hooks::window_scroll::use_window_scroll,
    player::Player,
    utils::into_either_of::IntoEitherOf7,
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

#[derive(Debug, Clone, Copy)]
struct GamePageRefs {
    pub cells: CellsRefs,
    pub chat: NodeRef<Div>,
    pub cell_popups: NodeRef<Div>,
}

impl GamePageRefs {
    pub fn provide_context(&self) {
        provide_context(*self);
    }

    pub fn use_context() -> Self {
        expect_context::<Self>()
    }
}

#[component]
pub fn GamePage() -> impl IntoView {
    let is_dice_shown = RwSignal::new(false);
    let game_state = GameState::new();
    let refs = GamePageRefs {
        cells: CellsRefs::new(),
        chat: NodeRef::<Div>::new(),
        cell_popups: NodeRef::<Div>::new(),
    };

    game_state.provide_context();
    refs.provide_context();

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
                <Rows />
                <Chat node_ref=refs.chat class="col-[2/11] row-[2/11]".to_owned() />
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
                    .map(|player| view! { <PlayerToken player /> })
                    .collect_view()
            }}
        </div>
        <div node_ref=refs.cell_popups />
        <GameFinished />
    }
}

#[derive(Debug, Clone, Copy)]
enum RowSide {
    Top,
    Right,
    Bottom,
    Left,
}

impl From<usize> for RowSide {
    #[track_caller]
    fn from(index: usize) -> Self {
        match index {
            0..10 => RowSide::Top,
            10..20 => RowSide::Right,
            20..30 => RowSide::Bottom,
            30..40 => RowSide::Left,
            _ => unreachable!("There is only {CELLS_COUNT} cells, silly. ᓚ₍ ^. .^₎"),
        }
    }
}

#[component]
fn Rows() -> impl IntoView {
    let game_page_refs = GamePageRefs::use_context();

    (0..40)
        .map(|i| {
            let row_side = i.into();
            let column = move || match row_side {
                RowSide::Top => i + 1,
                RowSide::Right => 11,
                RowSide::Bottom => 31 - i,
                RowSide::Left => 1,
            };

            let row = move || match row_side {
                RowSide::Top => 1,
                RowSide::Right => i - 9,
                RowSide::Bottom => 11,
                RowSide::Left => 41 - i,
            };

            view! {
                <Cell
                    node_ref=game_page_refs.cells.set(i)
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
    let game_page_refs = GamePageRefs::use_context();
    let game_state = GameState::use_context();
    let is_info_open = RwSignal::new(false);
    let current_cell = game_state.get_cell(index);
    let is_property = current_cell.try_unwrap_property().is_ok();
    let row_side = index.into();
    let cell_bg = move || {
        current_cell
            .try_unwrap_property()
            .ok()
            .and_then(|prop| prop.owner())
            .map(|owner| owner.color.get_cell_gradient())
            .unwrap_or("#fff")
    };
    let rent_bg = move || {
        current_cell
            .try_unwrap_property()
            .map(|prop| prop.data.group.color)
            .unwrap_or_default()
    };

    // This causes "Uncaught InternalError: too much recursion"
    // TODO: Report to Leptos
    //
    // view! {
    //     <>
    //         <div />
    //         {()}
    //     </>
    // }

    // This triggers "todo!()" at "tachys-0.1.0-beta/src/view/any_view.rs:344" if "style:..."
    // applied to the component itself
    // TODO: Report to Leptos
    //
    // view! {
    //     <>
    //         <div />
    //         <div />
    //     </>
    // }

    Effect::new(move |_| {
        if is_info_open.get() && game_page_refs.cell_popups.get().is_none() {
            tracing::warn!(
                "Unable to open PropertyInfo modal - Portal's mount point does not exists"
            );
        }
    });

    view! {
        <div
            node_ref=node_ref
            class=move || {
                tw!(
                    "relative p-1",
                    is_property => "cursor-pointer",
                    cell_bg() == "#fff" => "text-black",
                )
            }
            style:background=cell_bg
            on:click=move |_| is_info_open.update(|x| *x = !*x)
        >
            {match current_cell {
                Cell::Start => "Start".into_either_of_7a(),
                Cell::Jail => "Jail".into_either_of_7b(),
                Cell::FreeParking => "FreeParking".into_either_of_7c(),
                Cell::GoToJail => "GoToJail".into_either_of_7d(),
                Cell::Tax(tax) => format!("Tax: {}", tax).into_either_of_7e(),
                Cell::Chance => "Chance".into_either_of_7f(),
                Cell::Property(prop) => {
                    view! {
                        <>
                            {format!("Property: {}", prop.data.title)}
                            <div
                                style:background=rent_bg
                                class=tw!(
                                    "absolute leading-7 text-center text-white",
                                    match row_side {
                                        RowSide::Top => "left-0 -top-7 w-full h-7",
                                        RowSide::Right => "top-0 -right-7 h-full w-7 [writing-mode:vertical-lr]",
                                        RowSide::Bottom => "left-0 -bottom-7 w-full h-7",
                                        RowSide::Left => "top-0 -left-7 h-full w-7 [writing-mode:vertical-lr] rotate-180",
                                    }
                                )
                            >
                                {move || {
                                    if let (PropertyType::Utility { levels }, Some(owner)) = (
                                        prop.ty,
                                        prop.owner(),
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
                        .into_either_of_7g()
                }
            }}
            {move || {
                current_cell
                    .try_unwrap_property()
                    .ok()
                    .zip(game_page_refs.cell_popups.get())
                    .map(|(property, mount)| {
                        view! {
                            <Portal mount>
                                <PropertyInfo cell_node_ref=node_ref is_info_open property />
                            </Portal>
                        }
                    })
            }}
        </div>
    }
}

#[component]
pub fn PropertyInfo(
    cell_node_ref: NodeRef<Div>,
    is_info_open: RwSignal<bool>,
    property: Property,
) -> impl IntoView {
    let game_state = GameState::use_context();
    let game_page_refs = GamePageRefs::use_context();
    let node_ref = NodeRef::<Div>::new();
    let (scroll_x, scroll_y) = use_window_scroll();
    let coordinates = RwSignal::new(None::<(f64, f64)>);

    let handle = window_event_listener(ev::click, move |event| {
        // It would be cool if "NodeRef" implemented Fn family of traits.
        // TODO: PR into Leptos
        let Some((cell_node_ref, node_ref)) = cell_node_ref.get().zip(node_ref.get()) else {
            return;
        };

        let target = event_target::<Node>(&event);
        let is_click_inside_cell =
            cell_node_ref.contains(Some(&target)) || node_ref.contains(Some(&target));

        if !is_click_inside_cell {
            is_info_open.set(false)
        }
    });

    Effect::new(move |_| {
        let Some(((cell_node_ref, chat_ref), node_ref)) = cell_node_ref
            .get()
            .zip(game_page_refs.chat.get())
            .zip(node_ref.get())
        else {
            return;
        };

        let cell_rect = cell_node_ref.get_bounding_client_rect();
        let chat_rect = chat_ref.get_bounding_client_rect();
        let node_rect = node_ref.get_bounding_client_rect();

        let cell_x_center = scroll_x() + cell_rect.x() + cell_rect.width() / 2f64;
        let cell_y_center = scroll_y() + cell_rect.y() + cell_rect.height() / 2f64;
        let padding = 8.;

        coordinates.set(Some((
            cell_x_center.clamp(
                scroll_x() + chat_rect.left() + (node_rect.width() / 2.) + padding,
                scroll_x() + chat_rect.right() - (node_rect.width() / 2.) - padding,
            ),
            cell_y_center.clamp(
                scroll_y() + chat_rect.top() + (node_rect.height() / 2.) + padding,
                scroll_y() + chat_rect.bottom() - (node_rect.height() / 2.) - padding,
            ),
        )));
    });

    on_cleanup(move || handle.remove());

    view! {
        <Show when=is_info_open>
            <div
                node_ref=node_ref
                class="overflow-hidden absolute z-10 w-52 rounded-md -translate-x-1/2 -translate-y-1/2"
                style:left=move || coordinates().map(|(x, _)| format!("{x}px")).unwrap_or_default()
                style:top=move || coordinates().map(|(_, y)| format!("{y}px")).unwrap_or_default()
            >
                // When I do this - hella long error is being reported without any useful info.
                // TODO: PR fix into Leptos
                // style=(move || "", "")
                <div class="py-3 px-4 text-white" style:background=property.data.group.color>
                    <div class="text-lg font-bold">{property.data.title}</div>
                    <div class="-mt-0.5 text-xs font-bold opacity-75">
                        {property.data.group.title}
                    </div>
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
                    <Show
                        when=move || {
                            game_state.self_player == game_state.current_player()
                                && property.owner() == Some(game_state.self_player)
                        }
                        fallback=move || {
                            property
                                .owner()
                                .is_none()
                                .then(|| {
                                    view! {
                                        <button
                                            class="p-2 rounded border-2"
                                            on:click=move |_| {
                                                let _ = property.buy(&game_state.self_player);
                                            }
                                        >
                                            "[Debug] Buy"
                                        </button>
                                    }
                                })
                        }
                    >
                        <div class="flex gap-3">
                            {match property.ty {
                                PropertyType::Transport { .. } | PropertyType::Utility { .. } => {
                                    if property.mortgaged_for().is_some() {
                                        view! {
                                            <button
                                                class="p-2 rounded border-2"
                                                on:click=move |_| {
                                                    if let Err(error) = property.recover() {
                                                        match error {
                                                            OwnedPropertyError::NoOwner { .. } => {
                                                                unreachable!(
                                                                    "Property must have owner - we checked it in the Show above",
                                                                );
                                                            }
                                                            OwnedPropertyError::NotEnoughMoney { .. } => {
                                                                todo!("Show modal");
                                                            }
                                                        }
                                                    }
                                                }
                                            >
                                                "Recover"
                                            </button>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <button
                                                class="p-2 rounded border-2"
                                                on:click=move |_| {
                                                    property
                                                        .mortgage()
                                                        .expect(
                                                            "Property must have owner - we checked it in the Show above",
                                                        )
                                                }
                                            >
                                                "Mortgage"
                                            </button>
                                        }
                                            .into_any()
                                    }
                                }
                                PropertyType::Simple { levels, level, .. } => {
                                    if property.mortgaged_for().is_some() {
                                        view! {
                                            <button
                                                class="p-2 rounded border-2"
                                                on:click=move |_| {
                                                    if let Err(error) = property.recover() {
                                                        match error {
                                                            OwnedPropertyError::NoOwner { .. } => {
                                                                unreachable!(
                                                                    "Property must have owner - we checked it in the Show above",
                                                                );
                                                            }
                                                            OwnedPropertyError::NotEnoughMoney { .. } => {
                                                                todo!("Show modal");
                                                            }
                                                        }
                                                    }
                                                }
                                            >
                                                "Recover"
                                            </button>
                                        }
                                            .into_any()
                                    } else {
                                        (move || {
                                            let has_monopoly = game_state
                                                .has_monopoly_on(
                                                    &game_state.self_player,
                                                    &property.data.group,
                                                );
                                            let other_props = game_state
                                                .get_properties_by_group(&property.data.group)
                                                .into_iter()
                                                .filter(|prop| {
                                                    prop.owner()
                                                        .is_some_and(|owner| owner == game_state.self_player)
                                                })
                                                .filter_map(|prop| {
                                                    if let PropertyType::Simple { level, .. } = prop.ty {
                                                        Some((level, prop))
                                                    } else {
                                                        None
                                                    }
                                                })
                                                .collect::<Vec<_>>();
                                            view! {
                                                <>
                                                    <Show when={
                                                        let other_props = other_props.clone();
                                                        move || {
                                                            has_monopoly && level() < levels.len() - 1
                                                                && !other_props
                                                                    .iter()
                                                                    .any(|(x, p)| x() < level() || p.mortgaged_for().is_some())
                                                                && !game_state
                                                                    .get_properties_by_group(&property.data.group)
                                                                    .into_iter()
                                                                    .any(|prop| prop.is_agency_built())
                                                        }
                                                    }>
                                                        <button
                                                            class="p-2 rounded border-2"
                                                            on:click=move |_| {
                                                                if let Err(error) = property.build_agency(&game_state) {
                                                                    match error {
                                                                        BuildAgencyError::NoOwner { .. } => {
                                                                            unreachable!(
                                                                                "Property must have owner - we checked it in the Show above",
                                                                            );
                                                                        }
                                                                        BuildAgencyError::NotEnoughMoney { .. } => {
                                                                            todo!("Show modal")
                                                                        }
                                                                        BuildAgencyError::AlreadyBuilt { .. } => todo!("Show modal"),
                                                                    }
                                                                }
                                                            }
                                                        >
                                                            "Build agency"
                                                        </button>
                                                    </Show>

                                                    {if has_monopoly && level() > 0
                                                        && !other_props.iter().any(|(x, _)| x() > level())
                                                    {
                                                        view! {
                                                            <button
                                                                class="p-2 rounded border-2"
                                                                on:click=move |_| {
                                                                    if let Err(error) = property.sell_agency() {
                                                                        match error {
                                                                            SellAgencyError::NoOwner { .. } => {
                                                                                unreachable!(
                                                                                    "Property must have owner - we checked it in the Show above",
                                                                                );
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            >
                                                                "Sell agency"
                                                            </button>
                                                        }
                                                            .into_any()
                                                    } else if other_props.iter().all(|(x, _)| x() == 0) {
                                                        view! {
                                                            <button
                                                                class="p-2 rounded border-2"
                                                                on:click=move |_| {
                                                                    property
                                                                        .mortgage()
                                                                        .expect(
                                                                            "Property must have owner - we checked it in the Show above",
                                                                        );
                                                                }
                                                            >
                                                                "Mortgage"
                                                            </button>
                                                        }
                                                            .into_any()
                                                    } else {
                                                        ().into_any()
                                                    }}
                                                </>
                                            }
                                        })
                                            .into_any()
                                    }
                                }
                            }}
                        </div>
                    </Show>
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
        </Show>
    }
}

#[component]
pub fn PlayerCard(player: Player, #[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    let game_state = GameState::use_context();

    let is_current_player = move || game_state.current_player() == player;
    let is_self_player = game_state.self_player == player;
    let bg = move || {
        is_current_player()
            .then(|| player.color.get_player_gradient())
            .unwrap_or_default()
    };

    let surrender = move |_| {
        if is_self_player {
            game_state.surrender_player(&player);
        }
    };

    view! {
        <div
            style:background=bg
            class=move || {
                tw!(
                    "flex flex-col items-center justify-center p-3 bg-gray-900 transition-all",
                    is_current_player() => "scale-105",
                    is_self_player => "cursor-pointer",
                    class()
                )
            }
            on:click=surrender
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
fn PlayerToken(player: Player) -> impl IntoView {
    let game_page_refs = GamePageRefs::use_context();
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
        let current_cell = game_page_refs.cells.get(player.position());
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
pub fn Chat(
    node_ref: NodeRef<Div>,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let game_state = GameState::use_context();
    let roll_dice_pending = RwSignal::new(false);

    let roll_dice = move |_| {
        if roll_dice_pending() {
            return;
        };

        game_state.spawn_local_abortable(async move { game_state.roll_dice().await });
    };

    view! {
        <div
            node_ref=node_ref
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

#[component]
pub fn GameFinished() -> impl IntoView {
    let game_state = GameState::use_context();

    let left_players = move || {
        game_state
            .get_players()
            .into_values()
            .filter(|player| !player.has_lost())
            .collect::<Vec<_>>()
    };

    view! {
        <Show when=move || left_players().len() == 1>
            <div class="flex absolute top-0 left-0 z-20 flex-col justify-center items-center w-screen h-screen bg-black/50">
                <div>"Game Finished"</div>
                <div>"Winner: " {move || left_players()[0].name.get_value()}</div>
            </div>
        </Show>
    }
}
