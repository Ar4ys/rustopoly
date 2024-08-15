use std::fmt::Display;

use derive_more::derive::{
    Add, AddAssign, Constructor, Deref, Div, DivAssign, From, Mul, MulAssign, Neg, Not, Rem,
    RemAssign, Sub, SubAssign, TryUnwrap,
};
use leptos::prelude::*;
use snafu::prelude::*;

use crate::{
    components::in_game_modal::ModalResponse,
    game_state::GameState,
    player::{NotEnoughMoneyError, Player, PlayerId},
    utils::rand,
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
                if prop
                    .owner
                    .get_untracked()
                    .is_some_and(|owner| owner == current_player)
                {
                    // TODO: Log into chat: "Stepped on his own property"
                } else if let Some(rent) = untrack(|| prop.rent(game_state)) {
                    game_state
                        .in_game_modal_state
                        .one_button_async(
                            &format!("Oi! You owe this fine lad some moneh: {}$", rent),
                            "Pay moneh",
                        )
                        .await;

                    if let Err(error) = prop.pay_rent(&current_player, game_state) {
                        match error {
                            OwnedPropertyError::NoOwner { source } => {
                                tracing::warn!(
                                    "Tried to pay rent to property \"{}\" without owner.",
                                    source.property_title
                                );
                                // TODO: Add modal for user
                            }
                            OwnedPropertyError::NotEnoughMoney { source } => {
                                // Ideally, we should never reach here, because UI should force player to surrender,
                                // if they don't have enough money.
                                tracing::error!(
                                    "Player \"{}\" (id: {}) does not have enough money ({}$) to pay rent for property. Surrendering...",
                                    source.player_name,
                                    source.player_id,
                                    source.amount,
                                );
                                game_state.surrender_player(&current_player);
                                if let Some(owner) = prop.owner.get_untracked() {
                                    owner.deposit(rent);
                                }
                            }
                        }
                    };
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
                        if let Err(error) = prop.buy(&current_player) {
                            match error {
                                FreePropertyError::AlreadyOwned { source } => {
                                    tracing::warn!(
                                        "Tried to buy property \"{}\", that already has owner \"{}\" (id: {}).",
                                        source.property_title,
                                        source.owner_name,
                                        source.owner_id,
                                    );
                                    // TODO: Show user a modal
                                }
                                FreePropertyError::NotEnoughMoney { source } => {
                                    // Ideally, we should never reach here, because UI should stop player from buying,
                                    // if they don't have enough money.
                                    tracing::error!(
                                        "Player \"{}\" (id: {}) does not have enough money ({}$) to buy property.",
                                        source.player_name,
                                        source.player_id,
                                        source.amount,
                                    );
                                    // TODO: Show user a modal
                                }
                            }
                        }
                    }
                }
            }

            Cell::Chance => {
                let random_chance = rand::get_usize(0..=5);
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

                if you_get.is_positive() {
                    current_player.deposit(you_get);
                } else if let Err(error) = current_player.withdraw(-you_get) {
                    // Ideally, we should never reach here, because UI should force player to surrender,
                    // if they don't have enough money.
                    tracing::error!(
                        "Player \"{}\" (id: {}) does not have enough money ({}$) to pay for chance. Surrendering...",
                        error.player_name,
                        error.player_id,
                        error.amount,
                    );
                    game_state.surrender_player(&current_player);
                }
            }

            Cell::Tax(amount) => {
                game_state
                    .in_game_modal_state
                    .one_button_async(&format!("You owe me: {amount}$"), "Pay moneh")
                    .await;

                if let Err(error) = current_player.withdraw(*amount) {
                    // Ideally, we should never reach here, because UI should force player to surrender,
                    // if they don't have enough money.
                    tracing::error!(
                        "Player \"{}\" (id: {}) does not have enough money ({}$) to pay for chance. Surrendering...",
                        error.player_name,
                        error.player_id,
                        error.amount,
                    );
                    game_state.surrender_player(&current_player);
                }
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

#[derive(Debug, Snafu)]
pub enum OwnedPropertyError {
    #[snafu(transparent)]
    NoOwner { source: NoOwnerError },
    #[snafu(transparent)]
    NotEnoughMoney { source: NotEnoughMoneyError },
}

#[derive(Debug, Snafu)]
pub enum FreePropertyError {
    #[snafu(transparent)]
    AlreadyOwned { source: HasOwnerError },
    #[snafu(transparent)]
    NotEnoughMoney { source: NotEnoughMoneyError },
}

#[derive(Debug, Snafu)]
pub enum AgencyPropertyError {
    #[snafu(transparent)]
    NoOwner { source: NoOwnerError },
    #[snafu(transparent)]
    NotEnoughMoney { source: NotEnoughMoneyError },
    #[snafu(display("Property \"{property_title}\" is not a PropertyType::Simple"))]
    NotASimpleProperty { property_title: &'static str },
}

#[derive(Debug, Snafu)]
#[snafu(display("Property \"{property_title}\" does not have owner"))]
pub struct NoOwnerError {
    pub property_title: &'static str,
}

#[derive(Debug, Snafu)]
#[snafu(display(
    "Property \"{property_title}\" already has owner \"{owner_name}\" (id: {owner_id})"
))]
pub struct HasOwnerError {
    pub property_title: &'static str,
    pub owner_name: String,
    pub owner_id: PlayerId,
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

    pub fn remove_owner(&self) {
        self.owner.set(None);
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

    pub fn pay_rent(
        &self,
        player: &Player,
        game_state: &GameState,
    ) -> Result<(), OwnedPropertyError> {
        let Some((owner, rent)) = self
            .owner
            .get_untracked()
            .zip(untrack(|| self.rent(game_state)))
        else {
            return NoOwnerSnafu {
                property_title: self.data.title,
            }
            .fail()
            .map_err(Into::into);
        };

        player.withdraw(rent)?;
        owner.deposit(rent);
        Ok(())
    }

    pub fn buy(&self, player: &Player) -> Result<(), FreePropertyError> {
        if let Some(owner) = self.owner.get_untracked() {
            return HasOwnerSnafu {
                property_title: self.data.title,
                owner_id: owner.id,
                owner_name: owner.name.get_value(),
            }
            .fail()
            .map_err(Into::into);
        };

        player.withdraw(self.data.price)?;
        self.owner.set(Some(*player));
        Ok(())
    }

    pub fn mortgage(&self) -> Result<(), NoOwnerError> {
        let Some(owner) = self.owner.get_untracked() else {
            return NoOwnerSnafu {
                property_title: self.data.title,
            }
            .fail();
        };

        self.mortgaged_for.set(Some(15));
        owner.deposit(self.reward_for_mortgaging());
        Ok(())
    }

    pub fn recover(&self) -> Result<(), OwnedPropertyError> {
        let Some(owner) = self.owner.get_untracked() else {
            return NoOwnerSnafu {
                property_title: self.data.title,
            }
            .fail()
            .map_err(Into::into);
        };

        self.mortgaged_for.set(None);
        owner.withdraw(self.recovery_price())?;
        Ok(())
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

    pub fn build_agency(&self) -> Result<(), AgencyPropertyError> {
        let Some(owner) = self.owner.get_untracked() else {
            return NoOwnerSnafu {
                property_title: self.data.title,
            }
            .fail()
            .map_err(Into::into);
        };

        let PropertyType::Simple {
            level_price, level, ..
        } = self.ty
        else {
            return NotASimplePropertySnafu {
                property_title: self.data.title,
            }
            .fail();
        };

        owner.withdraw(level_price)?;
        level.update(|x| *x += 1);
        Ok(())
    }

    pub fn sell_agency(&self) -> Result<(), AgencyPropertyError> {
        let Some(owner) = self.owner.get_untracked() else {
            return NoOwnerSnafu {
                property_title: self.data.title,
            }
            .fail()
            .map_err(Into::into);
        };

        let PropertyType::Simple {
            level_price, level, ..
        } = self.ty
        else {
            return NotASimplePropertySnafu {
                property_title: self.data.title,
            }
            .fail();
        };

        owner.deposit(level_price);
        level.update(|x| *x -= 1);
        Ok(())
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
