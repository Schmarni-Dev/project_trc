use std::{fmt::Display, str::FromStr};

use bevy::{
    prelude::{Deref, DerefMut},
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

use crate::Pos3;

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub enum Maybe<T> {
    Some(T),
    #[default]
    None,
}

impl<T> Maybe<T> {
    pub fn unwrap(self) -> T {
        Option::from(self).unwrap()
    }
}

impl<T> From<Maybe<T>> for Option<T> {
    fn from(val: Maybe<T>) -> Self {
        match val {
            Maybe::None => None,
            Maybe::Some(data) => Some(data),
        }
    }
}
impl<T> From<Option<T>> for Maybe<T> {
    fn from(val: Option<T>) -> Self {
        match val {
            None => Maybe::None,
            Some(data) => Maybe::Some(data),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Deref, DerefMut, Reflect)]
pub struct Inventory {
    #[deref]
    pub inv: Vec<Maybe<Item>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Deref, DerefMut)]
pub struct TurtleInventory {
    pub selected_slot: u8,
    #[deref]
    pub inv: [Maybe<Item>; 16],
}
impl IntoIterator for Inventory {
    type Item = (u32, Maybe<Item>);
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        let mut data = Vec::new();
        self.inv.into_iter().zip(0u32..).for_each(|(item, index)| {
            data.push((index, item));
        });
        data.into_iter()
    }
}

impl Default for TurtleInventory {
    fn default() -> Self {
        TurtleInventory {
            selected_slot: 1,
            inv: [
                Maybe::None,
                Maybe::None,
                Maybe::None,
                Maybe::None,
                Maybe::None,
                Maybe::None,
                Maybe::None,
                Maybe::None,
                Maybe::None,
                Maybe::None,
                Maybe::None,
                Maybe::None,
                Maybe::None,
                Maybe::None,
                Maybe::None,
                Maybe::None,
            ],
        }
    }
}

impl IntoIterator for TurtleInventory {
    type Item = (u8, Maybe<Item>);
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        let mut data = Vec::new();
        self.inv.into_iter().zip(0u8..).for_each(|(item, index)| {
            data.push((index, item));
        });
        data.into_iter()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
pub struct Item {
    pub count: u32,
    pub name: String,
}

pub type TurtleIndexType = i32;

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Turtle {
    pub index: TurtleIndexType,
    pub name: String,
    pub inventory: Maybe<TurtleInventory>,
    pub position: Pos3,
    pub orientation: Orientation,
    pub fuel: i32,
    pub max_fuel: i32,
    pub is_online: bool,
    pub world: String,
}

impl PartialEq for Turtle {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

#[allow(dead_code)]
#[derive(PartialEq)]
pub enum TurnDir {
    Left,
    Right,
}

pub fn get_rotated_orientation(curr_orient: Orientation, dir: TurnDir) -> Orientation {
    match curr_orient {
        Orientation::North => {
            if dir == TurnDir::Left {
                Orientation::West
            } else {
                Orientation::East
            }
        }

        Orientation::East => {
            if dir == TurnDir::Left {
                Orientation::North
            } else {
                Orientation::South
            }
        }

        Orientation::South => {
            if dir == TurnDir::Left {
                Orientation::East
            } else {
                Orientation::West
            }
        }

        Orientation::West => {
            if dir == TurnDir::Left {
                Orientation::South
            } else {
                Orientation::North
            }
        }
    }
}
#[allow(dead_code)]
impl Turtle {
    pub fn new_dummy(
        index: TurtleIndexType,
        world: String,
        pos: Pos3,
        orientation: Orientation,
    ) -> Turtle {
        Turtle::new(
            index,
            String::default(),
            None,
            pos,
            orientation,
            0,
            0,
            false,
            world,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        index: TurtleIndexType,
        name: String,
        inventory: Option<TurtleInventory>,
        position: Pos3,
        orientation: Orientation,
        fuel: i32,
        max_fuel: i32,
        is_online: bool,
        world: String,
    ) -> Turtle {
        Turtle {
            index,
            name,
            inventory: inventory.into(),
            position,
            orientation,
            fuel,
            max_fuel,
            is_online,
            world,
        }
    }
    pub fn turn(&self, dir: TurnDir) -> Orientation {
        get_rotated_orientation(self.orientation, dir)
    }

    pub fn get_forward_vec(&self) -> Pos3 {
        self.orientation.get_forward_vec()
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum MoveDirection {
    Forward,
    Back,
    Up,
    Down,
    Left,
    Right,
}
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub enum Orientation {
    #[default]
    /// Towards -Z
    North,
    /// Towards +X
    East,
    /// Towards +Z
    South,
    /// Towards -X
    West,
}

impl Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Orientation as S;
        f.write_str(match self {
            S::North => "North",
            S::South => "South",
            S::West => "West",
            S::East => "East",
        })
    }
}

impl FromStr for Orientation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Orientation as S;
        match s {
            "North" => Ok(S::North),
            "South" => Ok(S::South),
            "West" => Ok(S::West),
            "East" => Ok(S::East),
            _ => Err("Invalid String".into()),
        }
    }
}

impl Orientation {
    pub fn get_forward_vec(&self) -> Pos3 {
        match self {
            Orientation::North => Pos3::new(0, 0, -1),
            Orientation::East => Pos3::new(1, 0, 0),
            Orientation::South => Pos3::new(0, 0, 1),
            Orientation::West => Pos3::new(-1, 0, 0),
        }
    }
}
