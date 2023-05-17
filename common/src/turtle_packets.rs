use crate::turtle::{self, Inventory, Maybe, TurtleIndexType};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BlockData {}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum T2SPackets {
    Moved {
        direction: turtle::MoveDirection,
    },
    Info {
        index: TurtleIndexType,
        name: String,
        inventory: Inventory,
        fuel: f32,
        max_fuel: i32,
    },
    Blocks {
        up: Maybe<BlockData>,
        down: Maybe<BlockData>,
        front: Maybe<BlockData>,
    },
}
#[derive(serde::Serialize, serde::Deserialize)]
pub enum S2TPackets {
    Move { direction: turtle::MoveDirection },
    GetInfo,
}
