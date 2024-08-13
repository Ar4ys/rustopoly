use std::fmt::Display;

use derive_more::derive::{
    Add, AddAssign, Constructor, Deref, Div, DivAssign, From, Mul, MulAssign, Neg, Not, Rem,
    RemAssign, Sub, SubAssign, TryUnwrap,
};
use leptos::prelude::*;

use crate::{
    components::in_game_modal::ModalResponse, game_state::GameState, player::Player, utils::rand,
};

pub const CELLS_COUNT: usize = 40;

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

impl Cell {
    pub async fn trigger(&self, game_state: &GameState) {
        let current_player = untrack(|| game_state.current_player());
        match self {
            Cell::Jail | Cell::FreeParking => {}
            Cell::Start => current_player.deposit(1000.into()),
            Cell::GoToJail => current_player.set_is_in_jail(true),
            Cell::Property(prop) => {
                if let Some(rent) = untrack(|| prop.rent(game_state)) {
                    game_state
                        .in_game_modal_state
                        .one_button_async(
                            &format!("Oi! You owe this fine lad some moneh: {}$", rent),
                            "Pay moneh",
                        )
                        .await;
                    prop.pay_rent(&current_player, game_state)
                } else {
                    let response = game_state
                        .in_game_modal_state
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
                        prop.buy(&current_player);
                    }
                }
            }

            Cell::Chance => {
                let random_chance = rand::get_usize(0..=6);
                let you_get = [500, 1000, 2000, -2000, -1000, -500].map(Money::new)[random_chance];
                game_state
                    .in_game_modal_state
                    .one_button_async(
                        &format!("Your chance is: {you_get}$"),
                        if you_get.is_negative() {
                            "Pay moneh"
                        } else {
                            "Get moneh"
                        },
                    )
                    .await;

                current_player.deposit(you_get);
            }

            Cell::Tax(amount) => {
                game_state
                    .in_game_modal_state
                    .one_button_async(&format!("You owe me: {amount}$"), "Pay moneh")
                    .await;
                current_player.withdraw(*amount);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Property {
    pub ty: PropertyType,
    pub data: PropertyData,
    owner: RwSignal<Option<Player>>,
    mortgaged_for: RwSignal<Option<usize>>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            mortgaged_for: RwSignal::new(None),
        }
    }

    pub fn reward_for_mortgaging(&self) -> Money {
        self.data.price / 2
    }

    pub fn recovery_price(&self) -> Money {
        self.data.price * 6 / 10
    }

    pub fn owner(&self) -> Option<Player> {
        self.owner.get()
    }

    pub fn mortgaged_for(&self) -> Option<usize> {
        self.mortgaged_for.get()
    }

    pub fn rent(&self, game_state: &GameState) -> Option<Money> {
        let owner = self.owner.get()?;
        if self.mortgaged_for().is_some() {
            return Some(0.into());
        }

        Some(match self.ty {
            PropertyType::Simple { levels, level, .. } => {
                let rent = levels[level.get()];
                let has_monopoly_on = game_state.has_monopoly_on(&owner, &self.data.group);

                if has_monopoly_on && level.get() == 0 {
                    rent * 2
                } else {
                    rent
                }
            }
            PropertyType::Transport { levels } => {
                let (owns, _) = game_state.has_from_group(&owner, &self.data.group);
                levels[owns - 1]
            }

            PropertyType::Utility { levels } => {
                let (dice1, dice2) = game_state
                    .rolled_dice()
                    .expect("Rolled dice should be present when calculating rent");
                let (owns, _) = game_state.has_from_group(&owner, &self.data.group);
                let rent = levels[owns - 1];

                rent * (dice1 + dice2) as i64
            }
        })
    }

    pub fn pay_rent(&self, player: &Player, game_state: &GameState) {
        if let Some((owner, rent)) = self
            .owner
            .get_untracked()
            .zip(untrack(|| self.rent(game_state)))
        {
            player.withdraw(rent);
            owner.deposit(rent);
        } else {
            tracing::warn!("Tried to pay rent to property without owner.")
        }
    }

    pub fn buy(&self, player: &Player) {
        if self.owner.get_untracked().is_some() {
            tracing::warn!("Tried to buy property with owner.")
        } else {
            player.withdraw(self.data.price);
            self.owner.set(Some(*player));
        }
    }

    pub fn mortgage(&self) {
        let Some(owner) = self.owner.get_untracked() else {
            tracing::warn!("Tried to mortgage property without owner.");
            return;
        };

        self.mortgaged_for.set(Some(15));
        owner.deposit(self.reward_for_mortgaging());
    }

    pub fn recover(&self) {
        let Some(owner) = self.owner.get_untracked() else {
            tracing::warn!("Tried to recover property without owner.");
            return;
        };

        self.mortgaged_for.set(None);
        owner.withdraw(self.recovery_price());
    }

    pub fn mortgage_tick(&self) {
        let Some(mut mortgaged_for) = self.mortgaged_for.get_untracked() else {
            return;
        };

        mortgaged_for = mortgaged_for.saturating_sub(1);

        if mortgaged_for == 0 {
            self.owner.set(None);
            self.mortgaged_for.set(None)
        } else {
            self.mortgaged_for.set(Some(mortgaged_for))
        }
    }

    pub fn build_agency(&self) {
        let Some(owner) = self.owner.get_untracked() else {
            tracing::warn!("Tried to build agency on property without owner.");
            return;
        };

        let PropertyType::Simple {
            level_price, level, ..
        } = self.ty
        else {
            tracing::warn!("Tried to build agency on property that is not PropertyType::Simple.");
            return;
        };

        owner.withdraw(level_price);
        level.update(|x| *x += 1);
    }

    pub fn sell_agency(&self) {
        let Some(owner) = self.owner.get_untracked() else {
            tracing::warn!("Tried to sell agency on property without owner.");
            return;
        };

        let PropertyType::Simple {
            level_price, level, ..
        } = self.ty
        else {
            tracing::warn!("Tried to sell agency on property that is not PropertyType::Simple.");
            return;
        };

        owner.deposit(level_price);
        level.update(|x| *x -= 1);
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
