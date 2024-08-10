use std::fmt::Display;

use derive_more::derive::{
    Add, AddAssign, Constructor, Deref, Div, DivAssign, From, Mul, MulAssign, Neg, Not, Rem,
    RemAssign, Sub, SubAssign,
};
use leptos::prelude::*;

use crate::{
    components::in_game_modal::{InGameModalState, ModalResponse},
    player::Player,
    utils::rand,
};

pub const CELLS_COUNT: usize = 40;

#[derive(Debug, Clone, Copy)]
pub enum Cell {
    Start,
    Jail,
    FreeParking,
    GoToJail,
    Property(Property),
    Chance,
    Tax(Money),
}

impl Cell {
    pub async fn trigger(&self, player: Player, in_game_modal: InGameModalState) {
        match self {
            Cell::Jail | Cell::FreeParking => {}
            Cell::Start => player.deposit(1000.into()),
            Cell::GoToJail => player.set_is_in_jail(true),
            Cell::Property(prop) => {
                if let Some(owner) = prop.owner.get_untracked() {
                    let rent = untrack(|| prop.rent());
                    in_game_modal
                        .one_button_async(
                            &format!("Oi! You owe this fine lad some moneh: {}$", rent),
                            "Pay moneh",
                        )
                        .await;
                    player.withdraw(rent);
                    owner.deposit(rent);
                } else {
                    let response = in_game_modal
                        .two_buttons_async(
                            &format!(
                                "Oi! You wanna buy this fine land? It's gonna cost ya {}$",
                                prop.data.price
                            ),
                            "Buy",
                            "Decline",
                        )
                        .await;

                    if let ModalResponse::Ok = response {
                        player.withdraw(prop.data.price);
                        prop.owner.set(Some(player));
                    }
                }
            }

            Cell::Chance => {
                let random_chance = rand::get_usize(0..=6);
                let you_get = [500, 1000, 2000, -2000, -1000, -500].map(Money::new)[random_chance];
                in_game_modal
                    .one_button_async(
                        &format!("Your chance is: {you_get}$"),
                        if you_get.is_negative() {
                            "Pay moneh"
                        } else {
                            "Get moneh"
                        },
                    )
                    .await;

                player.deposit(you_get);
            }

            Cell::Tax(amount) => {
                in_game_modal
                    .one_button_async(&format!("You owe me: {amount}$"), "Pay moneh")
                    .await;
                player.withdraw(*amount);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Property {
    pub ty: PropertyType,
    pub data: PropertyData,
    // TODO: Make private
    pub owner: RwSignal<Option<Player>>,
    mortgaged_until: RwSignal<Option<usize>>,
}

#[derive(Debug, Clone, Copy)]
pub struct PropertyData {
    pub title: &'static str,
    pub price: Money,
    pub group: PropertyGroup,
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

    pub fn reward_for_mortgaging(&self) -> Money {
        self.data.price / 2
    }

    pub fn recovery_price(&self) -> Money {
        self.data.price * 6 / 10
    }

    pub fn rent(&self) -> Money {
        match self.ty {
            // TODO Properly double base rent if monopoly
            PropertyType::Simple { levels, level, .. } => levels[level.get()],
            // TODO Properly calculate rent
            PropertyType::Transport { levels } => levels[0],
            // TODO Properly calculate rent
            PropertyType::Utility { levels } => levels[0],
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
            write!(f, "{},{:03}", self.0 / 1000, (self.0 % 1000).abs())
        } else {
            write!(f, "{}", self.0)
        }
    }
}
