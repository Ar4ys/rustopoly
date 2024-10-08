use leptos::prelude::*;
use snafu::prelude::*;

use crate::cell::{Money, CELLS_COUNT};

#[derive(Debug, Clone, Copy)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
}

pub type PlayerId = u64;

#[derive(Debug, Clone, Copy)]
pub enum PlayerColor {
    Red,
    Blue,
    Green,
    Purple,
    Yellow,
}

impl PlayerColor {
    pub fn get_player_gradient(&self) -> &'static str {
        match self {
            PlayerColor::Red => "linear-gradient(45deg,#cd3747,#f26b61)",
            PlayerColor::Blue => "linear-gradient(45deg,#54c9f0,#2191e1)",
            PlayerColor::Green => "linear-gradient(45deg,#66b343,#b0e372)",
            PlayerColor::Purple => "linear-gradient(45deg,#a17fef,#d188e3)",
            PlayerColor::Yellow => todo!("Add player card gradient to yellow"),
        }
    }

    pub fn get_cell_gradient(&self) -> &'static str {
        match self {
            PlayerColor::Red => "linear-gradient(45deg,#d96975,#f59088)",
            PlayerColor::Blue => "linear-gradient(45deg,#7ed6f3,#58ace8)",
            PlayerColor::Green => "linear-gradient(45deg,#8cc672,#c3ea95)",
            PlayerColor::Purple => "linear-gradient(45deg,#b89ff3,#dca5ea)",
            PlayerColor::Yellow => todo!("Add cell gradient to yellow"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub id: PlayerId,
    pub name: StoredValue<String>,
    pub color: PlayerColor,
    balance: RwSignal<Money>,
    position: RwSignal<usize>,
    pub is_in_jail: RwSignal<bool>,
    has_lost: RwSignal<bool>,
    connection_status: RwSignal<ConnectionStatus>,
}

#[derive(Debug, Snafu)]
#[snafu(display(
    "Player \"{player_name}\" (id: {player_id}) does not have enough money ({amount}$)"
))]
pub struct NotEnoughMoneyError {
    pub player_name: String,
    pub player_id: PlayerId,
    pub amount: Money,
}

impl Player {
    pub fn new(id: PlayerId, name: &str, color: PlayerColor) -> Self {
        Self {
            id,
            name: StoredValue::new(name.to_owned()),
            color,
            balance: RwSignal::new(15_000.into()),
            position: RwSignal::new(0),
            is_in_jail: RwSignal::new(false),
            has_lost: RwSignal::new(false),
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

    pub fn append_position(&self, index: usize) -> (usize, usize) {
        self.position
            .try_update(|position| {
                let prev = *position;
                *position = (*position + index) % CELLS_COUNT;
                (prev, *position)
            })
            .expect("Player::position signal should not be disposed")
    }

    pub fn position(&self) -> usize {
        self.position.get()
    }

    pub fn balance(&self) -> Money {
        self.balance.get()
    }

    pub fn deposit(&self, amount: Money) {
        self.balance.update(|balance| *balance += amount)
    }

    pub fn withdraw(&self, amount: Money) -> Result<(), NotEnoughMoneyError> {
        ensure!(
            self.balance.get_untracked() >= amount,
            NotEnoughMoneySnafu {
                player_id: self.id,
                player_name: self.name.get_value(),
                amount
            }
        );

        self.balance.update(|balance| *balance -= amount);

        Ok(())
    }

    pub fn is_in_jail(&self) -> bool {
        self.is_in_jail.get()
    }

    pub fn set_is_in_jail(&self, state: bool) {
        self.is_in_jail.set(state);
        if state {
            self.set_position(10);
        }
    }

    pub fn has_lost(&self) -> bool {
        self.has_lost.get()
    }

    pub fn surrender(&self) {
        self.has_lost.set(true);
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Player {}
