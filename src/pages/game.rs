use std::convert::identity;

use leptos::{html::Div, prelude::*};
use tailwind_merge::tw;
use web_sys::HtmlDivElement;

use crate::{
    game_state::{Cell, GameState, CELLS_COUNT},
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
        <div class="game-grid min-h-full aspect-square">
            <Rows cells_refs/>
            <div class="col-[2/11] row-[2/11] bg-cyan-700">"Chat"</div>
        </div>
        <PlayerCursor cells_refs/>
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
    let bg_class = move || match current_cell() {
        Cell::Start => "bg-green-500",
        Cell::Jail => "bg-yellow-500",
        Cell::FreeParking => "bg-red-500",
        Cell::GoToJail => "bg-purple-500",
        Cell::Property => "bg-gray-500",
    };

    view! {
        <div
            node_ref=node_ref
            class=move || tw!("p-1 cursor-pointer", bg_class())
            on:click=move |_| game_state.set_position(index)
        >
            {match current_cell() {
                Cell::Start => "Start",
                Cell::Jail => "Jail",
                Cell::FreeParking => "FreeParking",
                Cell::GoToJail => "GoToJail",
                Cell::Property => "Property",
            }}

        </div>
    }
}

#[component]
fn PlayerCursor(cells_refs: CellsRefs) -> impl IntoView {
    let width = 12f64;
    let height = 12f64;

    let game_state = GameState::use_context();
    let (scroll_x, scroll_y) = use_window_scroll();
    let transform = RwSignal::new(String::new());

    Effect::new(move |_| {
        // TODO: Calculate position if there is more than one player on a cell
        let current_cell = cells_refs.get(game_state.position());
        let cell_rect = current_cell.get_bounding_client_rect();
        let x = scroll_x() + cell_rect.x() + cell_rect.width() / 2f64 - width / 2f64;
        let y = scroll_y() + cell_rect.y() + cell_rect.height() / 2f64 - height / 2f64;

        transform.set(format!("translate({x}px, {y}px)"));
    });

    view! {
        <div
            style:transform=transform
            style:width=width.to_string()
            style:height=height.to_string()
            class=move || {
                tw!(
                    "absolute left-0 top-0 w-3 h-3 bg-red-400 border-gray-600 border transition-transform",
                    transform().is_empty() => "hidden"
                )
            }
        >
        </div>
    }
}
