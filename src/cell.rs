use std::fmt::Display;

use derive_more::derive::{
    Add, AddAssign, Constructor, Deref, Div, DivAssign, From, Mul, MulAssign, Neg, Not, Rem,
    RemAssign, Sub, SubAssign,
};
use leptos::prelude::*;

use crate::player::Player;

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
