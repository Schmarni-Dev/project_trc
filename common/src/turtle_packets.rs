use crate::turtle::{self, Inventory, Maybe, TurtleIndexType};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct InfoData {
    pub index: TurtleIndexType,
    pub name: String,
    pub inventory: Inventory,
    pub fuel: f32,
    pub max_fuel: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum T2SPackets {
    Moved {
        direction: turtle::MoveDirection,
    },
    Info(InfoData),
    Blocks {
        up: Maybe<String>,
        down: Maybe<String>,
        front: Maybe<String>,
    },
}
#[derive(serde::Serialize, serde::Deserialize)]
pub enum S2TPackets {
    Move { direction: turtle::MoveDirection },
    GetInfo,
}
