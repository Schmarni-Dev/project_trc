use crate::turtle::{BlockDirection, MoveDirection};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum RcT2SPacket {}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum RcS2TPacket {
    Move(Vec<MoveDirection>),
    SelectSlot(u32),
    PlaceBlock {
        dir: BlockDirection,
        text: Option<String>,
    },
    BreakBlock {
        dir: BlockDirection,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum RcS2CPacket {}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum RcC2SPacket {
    MoveTurtle {
        index: i32,
        world: String,
        direction: MoveDirection,
    },
    TurtleSelectSlot {
        index: i32,
        world: String,
        slot: u32,
    },
    PlaceBlock {
        index: i32,
        world: String,
        dir: BlockDirection,
        text: Option<String>,
    },
    BreakBlock {
        index: i32,
        world: String,
        dir: BlockDirection,
    },
}

