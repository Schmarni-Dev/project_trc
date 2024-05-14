use crate::turtle::MoveDirection;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum T2SPackets {}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum S2TPackets {
    Move(Vec<MoveDirection>),
    SelectSlot(u32),
    PlaceBlock {
        dir: TurtleUpDown,
        text: Option<String>,
    },
    BreakBlock {
        dir: TurtleUpDown,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum S2CPacket {}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum C2SPacket {
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
        dir: TurtleUpDown,
        text: Option<String>,
    },
    BreakBlock {
        index: i32,
        world: String,
        dir: TurtleUpDown,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum TurtleUpDown {
    Up,
    Forward,
    Down,
}
