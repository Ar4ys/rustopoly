use leptos::prelude::RwSignal;

use crate::cell::{Cell, Money, Property, PropertyData, PropertyGroup, PropertyType, CELLS_COUNT};

// The names and colors don't actually match, because the naming scheme is from
// the original Monopoly, while the colors are from Monopoly One
const BROWN_GROUP: PropertyGroup = PropertyGroup {
    title: "Perfumes",
    color: "#ec87c1",
};

const LIGHT_BLUE_GROUP: PropertyGroup = PropertyGroup {
    title: "Clothing",
    color: "#e0b439",
};

const PINK_GROUP: PropertyGroup = PropertyGroup {
    title: "Web Services",
    color: "#37bc9d",
};

const ORANGE_GROUP: PropertyGroup = PropertyGroup {
    title: "Drinks",
    color: "#4b89dc",
};

const RED_GROUP: PropertyGroup = PropertyGroup {
    title: "Airlines",
    color: "#8cc152",
};

const YELLOW_GROUP: PropertyGroup = PropertyGroup {
    title: "Restaurants",
    color: "#4fc1e9",
};

const GREEN_GROUP: PropertyGroup = PropertyGroup {
    title: "Hotels",
    color: "#967bdc",
};

const DARK_BLUE_GROUP: PropertyGroup = PropertyGroup {
    title: "Electronics",
    color: "#656d78",
};

const TRANSPORT_GROUP: PropertyGroup = PropertyGroup {
    title: "Autos",
    color: "#da4553",
};

const TRANSPORT_PROPERTY_TYPE: PropertyType = PropertyType::Transport {
    levels: [
        Money::new(250),
        Money::new(500),
        Money::new(1000),
        Money::new(2000),
    ],
};

const UTILITIES_GROUP: PropertyGroup = PropertyGroup {
    title: "Game Developers",
    color: "#7f1f0f",
};

const UTILITIES_PROPERTY_TYPE: PropertyType = PropertyType::Utility {
    levels: [Money::new(100), Money::new(250)],
};

pub fn init_cells() -> [Cell; CELLS_COUNT] {
    [
        Cell::Start,
        Cell::Property(Property::new(
            PropertyData {
                title: "Chanel",
                price: 600.into(),
                group: BROWN_GROUP,
            },
            PropertyType::Simple {
                levels: [20, 100, 300, 900, 1600, 2500].map(Into::into),
                level_price: 500.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Chance,
        Cell::Property(Property::new(
            PropertyData {
                title: "Hugo Boss",
                price: 600.into(),
                group: BROWN_GROUP,
            },
            PropertyType::Simple {
                levels: [40, 200, 600, 1800, 3200, 4500].map(Into::into),
                level_price: 500.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Tax(2000.into()),
        Cell::Property(Property::new(
            PropertyData {
                title: "Mercedes",
                price: 2000.into(),
                group: TRANSPORT_GROUP,
            },
            TRANSPORT_PROPERTY_TYPE,
        )),
        Cell::Property(Property::new(
            PropertyData {
                title: "Adidas",
                price: 1000.into(),
                group: LIGHT_BLUE_GROUP,
            },
            PropertyType::Simple {
                levels: [60, 300, 900, 2700, 4000, 5500].map(Into::into),
                level_price: 500.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Chance,
        Cell::Property(Property::new(
            PropertyData {
                title: "Puma",
                price: 1000.into(),
                group: LIGHT_BLUE_GROUP,
            },
            PropertyType::Simple {
                levels: [60, 300, 900, 2700, 4000, 5500].map(Into::into),
                level_price: 500.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Property(Property::new(
            PropertyData {
                title: "Lacoste",
                price: 1200.into(),
                group: LIGHT_BLUE_GROUP,
            },
            PropertyType::Simple {
                levels: [80, 400, 1000, 3000, 4500, 6000].map(Into::into),
                level_price: 500.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Jail,
        Cell::Property(Property::new(
            PropertyData {
                title: "VK",
                price: 1400.into(),
                group: PINK_GROUP,
            },
            PropertyType::Simple {
                levels: [100, 500, 1500, 4500, 6250, 7500].map(Into::into),
                level_price: 750.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Property(Property::new(
            PropertyData {
                title: "Rockstar Games",
                price: 1500.into(),
                group: UTILITIES_GROUP,
            },
            UTILITIES_PROPERTY_TYPE,
        )),
        Cell::Property(Property::new(
            PropertyData {
                title: "Facebook",
                price: 1400.into(),
                group: PINK_GROUP,
            },
            PropertyType::Simple {
                levels: [100, 500, 1500, 4500, 6250, 7500].map(Into::into),
                level_price: 750.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Property(Property::new(
            PropertyData {
                title: "Twitter",
                price: 1600.into(),
                group: PINK_GROUP,
            },
            PropertyType::Simple {
                levels: [120, 600, 1800, 5000, 7000, 9000].map(Into::into),
                level_price: 750.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Property(Property::new(
            PropertyData {
                title: "Audi",
                price: 2000.into(),
                group: TRANSPORT_GROUP,
            },
            TRANSPORT_PROPERTY_TYPE,
        )),
        Cell::Property(Property::new(
            PropertyData {
                title: "Coca-Cola",
                price: 1800.into(),
                group: ORANGE_GROUP,
            },
            PropertyType::Simple {
                levels: [140, 700, 2000, 5500, 7500, 9500].map(Into::into),
                level_price: 1000.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Chance,
        Cell::Property(Property::new(
            PropertyData {
                title: "Pepsi",
                price: 1800.into(),
                group: ORANGE_GROUP,
            },
            PropertyType::Simple {
                levels: [140, 700, 2000, 5500, 7500, 9500].map(Into::into),
                level_price: 1000.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Property(Property::new(
            PropertyData {
                title: "Fanta",
                price: 2000.into(),
                group: ORANGE_GROUP,
            },
            PropertyType::Simple {
                levels: [160, 800, 2200, 6000, 8000, 10000].map(Into::into),
                level_price: 1000.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::FreeParking,
        Cell::Property(Property::new(
            PropertyData {
                title: "American Airlines",
                price: 2200.into(),
                group: RED_GROUP,
            },
            PropertyType::Simple {
                levels: [180, 900, 2500, 7000, 8750, 10500].map(Into::into),
                level_price: 1250.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Chance,
        Cell::Property(Property::new(
            PropertyData {
                title: "Lufthansa",
                price: 2200.into(),
                group: RED_GROUP,
            },
            PropertyType::Simple {
                levels: [180, 900, 2500, 7000, 8750, 10500].map(Into::into),
                level_price: 1250.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Property(Property::new(
            PropertyData {
                title: "British Airways",
                price: 2400.into(),
                group: RED_GROUP,
            },
            PropertyType::Simple {
                levels: [200, 1000, 3000, 7500, 9250, 11000].map(Into::into),
                level_price: 1250.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Property(Property::new(
            PropertyData {
                title: "Ford",
                price: 2000.into(),
                group: TRANSPORT_GROUP,
            },
            TRANSPORT_PROPERTY_TYPE,
        )),
        Cell::Property(Property::new(
            PropertyData {
                title: "McDonald's",
                price: 2600.into(),
                group: YELLOW_GROUP,
            },
            PropertyType::Simple {
                levels: [220, 1100, 3300, 8000, 9750, 1150].map(Into::into),
                level_price: 1500.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Property(Property::new(
            PropertyData {
                title: "Burger King",
                price: 2600.into(),
                group: YELLOW_GROUP,
            },
            PropertyType::Simple {
                levels: [220, 1100, 3300, 8000, 9750, 1150].map(Into::into),
                level_price: 1500.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Property(Property::new(
            PropertyData {
                title: "Rovio",
                price: 1500.into(),
                group: UTILITIES_GROUP,
            },
            UTILITIES_PROPERTY_TYPE,
        )),
        Cell::Property(Property::new(
            PropertyData {
                title: "KFC",
                price: 2800.into(),
                group: YELLOW_GROUP,
            },
            PropertyType::Simple {
                levels: [240, 1200, 3600, 8500, 10250, 12000].map(Into::into),
                level_price: 1500.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::GoToJail,
        Cell::Property(Property::new(
            PropertyData {
                title: "Holiday Inn",
                price: 3000.into(),
                group: GREEN_GROUP,
            },
            PropertyType::Simple {
                levels: [260, 1300, 3900, 9000, 11000, 12750].map(Into::into),
                level_price: 1750.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Property(Property::new(
            PropertyData {
                title: "Radisson Blu",
                price: 3000.into(),
                group: GREEN_GROUP,
            },
            PropertyType::Simple {
                levels: [260, 1300, 3900, 9000, 11000, 12750].map(Into::into),
                level_price: 1750.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Chance,
        Cell::Property(Property::new(
            PropertyData {
                title: "Novotel",
                price: 3200.into(),
                group: GREEN_GROUP,
            },
            PropertyType::Simple {
                levels: [280, 1500, 4500, 10000, 12000, 14000].map(Into::into),
                level_price: 1750.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Property(Property::new(
            PropertyData {
                title: "Land Rover",
                price: 2000.into(),
                group: TRANSPORT_GROUP,
            },
            TRANSPORT_PROPERTY_TYPE,
        )),
        Cell::Tax(1000.into()),
        Cell::Property(Property::new(
            PropertyData {
                title: "Apple",
                price: 3500.into(),
                group: DARK_BLUE_GROUP,
            },
            PropertyType::Simple {
                levels: [350, 1750, 5000, 11000, 13000, 15000].map(Into::into),
                level_price: 2000.into(),
                level: RwSignal::new(0),
            },
        )),
        Cell::Chance,
        Cell::Property(Property::new(
            PropertyData {
                title: "Nokia",
                price: 4000.into(),
                group: DARK_BLUE_GROUP,
            },
            PropertyType::Simple {
                levels: [500, 2000, 6000, 14000, 17000, 20000].map(Into::into),
                level_price: 2000.into(),
                level: RwSignal::new(0),
            },
        )),
    ]
}
