use std::{collections::HashMap, fmt::Display};

use derive_more::derive::{
    Add, AddAssign, Constructor, Deref, Div, DivAssign, From, Mul, MulAssign, Neg, Not, Rem,
    RemAssign, Sub, SubAssign, TryUnwrap,
};
use leptos::prelude::*;

use crate::{
    game_data::init_cells,
    utils::{oneshot_event_emitter::OneShotEventEmitter, rand},
};

pub const CELLS_COUNT: usize = 40;

#[derive(Debug, Clone, Copy)]
pub struct GameState {
    cells: [Cell; CELLS_COUNT],
    players: RwSignal<HashMap<PlayerId, Player>>,
    pub self_player: Player,
    current_player: RwSignal<Player>,
    current_step: RwSignal<usize>,
    current_round: RwSignal<usize>,
    render_dice: RwSignal<Option<(usize, usize)>>,
    player_token_transition_end: OneShotEventEmitter,
    dice_transition_end: OneShotEventEmitter,
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
            cells: init_cells(),
            players: RwSignal::new(players),
            render_dice: RwSignal::new(None),
            player_token_transition_end: OneShotEventEmitter::new(),
            dice_transition_end: OneShotEventEmitter::new(),
        }
    }

    pub fn use_context() -> Self {
        expect_context::<Self>()
    }

    pub fn current_player(&self) -> Player {
        self.current_player.get()
    }

    pub fn render_dice(&self) -> Option<(usize, usize)> {
        self.render_dice.get()
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
                .values()
                .filter_map(|player| (player.position() == index).then_some(*player))
                .collect()
        })
    }

    pub async fn roll_dice(&self) {
        let dice1 = rand::get_usize(1..=6);
        let dice2 = rand::get_usize(1..=6);
        self.render_dice.set(Some((dice1, dice2)));
        self.dice_transition_end.listen_async().await;
        self.current_player
            .with_untracked(|player| player.append_position(dice1 + dice2));

        self.player_token_transition_end.listen_async().await;
        self.render_dice.set(None);
        self.finish_step();
    }

    pub fn finish_step(&self) {
        let is_round_ended = untrack(|| self.next_player());
        self.current_step.update(|step| *step += 1);
        if is_round_ended {
            self.current_round.update(|round| *round += 1);
        }
    }

    pub fn next_player(&self) -> bool {
        let players_left: Vec<_> = self.players.with(|players| {
            players
                .values()
                // Skip all players before current one
                .skip_while(|player| **player != self.current_player())
                // Skip current player
                .skip(1)
                .copied()
                .collect()
        });

        let (next_player, is_round_ended) = if players_left.is_empty() {
            (
                self.players.with(|players| {
                    players
                        .values()
                        .next()
                        .copied()
                        .expect("There should be players!")
                }),
                true,
            )
        } else {
            (players_left[0], false)
        };

        self.current_player.set(next_player);

        is_round_ended
    }

    pub fn player_token_transition_end(&self) {
        self.player_token_transition_end.trigger();
    }
    pub fn dice_transition_end(&self) {
        self.dice_transition_end.trigger();
    }
}

#[derive(Debug, Clone, Copy, TryUnwrap)]
pub enum Cell {
    Start,
    Jail,
    FreeParking,
    GoToJail,
    Property(Property),
    Chance,
    Tax(Money),
}

#[derive(Debug, Clone, Copy)]
pub struct Property {
    pub ty: PropertyType,
    pub data: PropertyData,
    owner: RwSignal<Option<Player>>,
    mortgaged_until: RwSignal<Option<usize>>,
}

#[derive(Debug, Clone, Copy)]
pub struct PropertyData {
    pub title: &'static str,
    pub price: Money,
    pub group: PropertyGroup,
}

impl PropertyData {
    fn reward_for_mortgaging(&self) -> Money {
        self.price / 2
    }

    fn recovery_price(&self) -> Money {
        self.price * 6 / 10
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PropertyType {
    Simple {
        levels: [Money; 6],
        level_price: Money,
        level: RwSignal<usize>,
    },

    Transport {
        levels: [Money; 4],
    },

    Utility {
        levels: [Money; 2],
    },
}

#[derive(Debug, Clone, Copy)]
pub struct PropertyGroup {
    pub title: &'static str,
    pub color: &'static str,
}

impl Property {
    pub fn new(data: PropertyData, ty: PropertyType) -> Self {
        Self {
            ty,
            data,
            owner: RwSignal::new(None),
            mortgaged_until: RwSignal::new(None),
        }
    }
}

#[derive(
    Debug,
    Default,
    Constructor,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    From,
    Deref,
    Not,
    Neg,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
)]
pub struct Money(i64);

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 >= 1000 {
            write!(f, "{},{:03}", self.0 / 1000, self.0 % 1000)
        } else {
            write!(f, "{}", self.0)
        }
    }
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

    pub fn append_position(&self, index: usize) {
        self.position
            .update(|position| *position = (*position + index) % CELLS_COUNT)
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
