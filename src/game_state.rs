use leptos::prelude::*;

use crate::game_data::CELLS;

pub const CELLS_COUNT: usize = 40;

#[derive(Debug, Clone, Copy)]
pub struct GameState {
    position: RwSignal<usize>,
    cells: [RwSignal<Cell>; CELLS_COUNT],
}

impl GameState {
    pub fn new() -> Self {
        Self {
            position: RwSignal::new(0),
            cells: CELLS.map(RwSignal::new),
        }
    }

    pub fn use_context() -> Self {
        expect_context::<Self>()
    }

    pub fn set_position(&self, index: usize) {
        assert!(
            index < CELLS_COUNT,
            "There is only {CELLS_COUNT} cells, dummy. Provided index: {index}"
        );
        self.position.set(index)
    }

    pub fn position(&self) -> usize {
        (self.position)()
    }

    pub fn get_cell(&self, index: usize) -> Cell {
        self.cells[index].get()
    }
}

#[derive(Debug, Clone)]
pub enum Cell {
    Start,
    Jail,
    FreeParking,
    GoToJail,
    Property,
}
