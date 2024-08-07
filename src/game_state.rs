use std::collections::HashMap;

use leptos::prelude::*;

use crate::game_data::CELLS;

pub const CELLS_COUNT: usize = 40;

#[derive(Debug, Clone, Copy)]
pub struct GameState {
    cells: [Cell; CELLS_COUNT],
    players: RwSignal<HashMap<PlayerId, Player>>,
    pub self_player: Player,
    current_player: RwSignal<Player>,
    current_step: RwSignal<usize>,
    current_round: RwSignal<usize>,
}

impl GameState {
    pub fn new() -> Self {
        let mut players = HashMap::new();
        players.insert(0, Player::new(0, "Ar4ys", "#f87171"));
        players.insert(1, Player::new(1, "Madeline", "#bfa0f1"));

        Self {
            self_player: players[&0],
            current_player: RwSignal::new(players[&0]),
            current_step: RwSignal::new(0),
            current_round: RwSignal::new(0),
            cells: CELLS.map(Cell::new),
            players: RwSignal::new(players),
        }
    }

    pub fn use_context() -> Self {
        expect_context::<Self>()
    }

    pub fn current_player(&self) -> Player {
        self.current_player.get()
    }

    pub fn get_cell(&self, index: usize) -> Cell {
        self.cells[index]
    }

    pub fn get_players(&self) -> HashMap<PlayerId, Player> {
        self.players.get()
    }

    pub fn get_player_by_id(&self, id: PlayerId) -> Player {
        self.players.with_untracked(|players| {
            *players
                .get(&id)
                .unwrap_or_else(|| panic!("Player with id \"{id}\" should to exists"))
        })
    }

    pub fn get_players_by_cell(&self, index: usize) -> Vec<Player> {
        self.players.with(|players| {
            players
                .iter()
                .filter_map(|(_, player)| (player.position() == index).then_some(*player))
                .collect()
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub ty: CellType,
    state: CellState,
}

impl Cell {
    pub fn new(ty: CellType) -> Self {
        Self {
            ty,
            state: CellState::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CellState {
    owner: RwSignal<Option<Player>>,
    level: RwSignal<u8>,
}

impl CellState {
    pub fn new() -> Self {
        Self {
            owner: RwSignal::new(None),
            level: RwSignal::new(0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CellType {
    Start,
    Jail,
    FreeParking,
    GoToJail,
    Property,
}

#[derive(Debug, Clone, Copy)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
}

type PlayerId = u64;

#[derive(Debug, Clone, Copy)]
pub enum PlayerState {
    Won,
    Lost,
    Playing,
    Left,
}

#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub id: PlayerId,
    pub name: StoredValue<String>,
    pub color: StoredValue<String>,
    balance: RwSignal<i64>,
    position: RwSignal<usize>,
    state: RwSignal<PlayerState>,
    connection_status: RwSignal<ConnectionStatus>,
}

impl Player {
    pub fn new(id: PlayerId, name: &str, color: &str) -> Self {
        Self {
            id,
            name: StoredValue::new(name.to_owned()),
            color: StoredValue::new(color.to_owned()),
            balance: RwSignal::new(0),
            position: RwSignal::new(0),
            state: RwSignal::new(PlayerState::Playing),
            connection_status: RwSignal::new(ConnectionStatus::Connected),
        }
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
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Player {}
