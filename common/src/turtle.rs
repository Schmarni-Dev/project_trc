use std::str::FromStr;

use bevy::{
    prelude::{Deref, DerefMut},
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

use crate::Pos3;

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
pub enum Maybe<T> {
    Some(T),
    None,
}

impl<T> From<Maybe<T>> for Option<T> {
    fn from(val: Maybe<T>) -> Self {
        match val {
            Maybe::None => None,
            Maybe::Some(data) => Some(data),
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
    pub inventory: TurtleInventory,
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
            TurtleInventory::default(),
            pos,
            orientation,
            0,
            0,
            false,
            world,
        )
    }
    pub fn new(
        index: TurtleIndexType,
        name: String,
        inventory: TurtleInventory,
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
            inventory,
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

impl ToString for Orientation {
    fn to_string(&self) -> String {
        use Orientation as S;
        match self {
            S::North => "North".into(),
            S::South => "South".into(),
            S::West => "West".into(),
            S::East => "East".into(),
        }
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
