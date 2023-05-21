use serde::{Deserialize, Serialize};

use crate::Pos3;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Maybe<T> {
    Some(T),
    None,
}

impl<T> Into<Option<T>> for Maybe<T> {
    fn into(self) -> Option<T> {
        match self {
            Maybe::None => None,
            Maybe::Some(data) => Some(data),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Inventory {
    inv: [Maybe<Item>; 16],
}

impl Default for Inventory {
    fn default() -> Self {
        Inventory {
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

impl IntoIterator for Inventory {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    count: u8,
    name: String,
}

pub type TurtleIndexType = i32;

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Turtle {
    pub index: TurtleIndexType,
    pub name: String,
    pub inventory: Inventory,
    pub position: Pos3,
    pub orientation: Orientation,
    pub fuel: f32,
    pub max_fuel: i32,
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

#[allow(dead_code)]
impl Turtle {
    pub fn new(
        index: TurtleIndexType,
        name: String,
        inventory: Inventory,
        position: Pos3,
        orientation: Orientation,
        fuel: f32,
        max_fuel: i32,
    ) -> Turtle {
        Turtle {
            index,
            name,
            inventory,
            position,
            orientation,
            fuel,
            max_fuel,
        }
    }
    pub fn turn(&self, dir: TurnDir) -> Orientation {
        match self.orientation {
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
