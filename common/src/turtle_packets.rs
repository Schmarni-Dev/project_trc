use crate::{
    turtle::{TurtleInventory, Maybe, MoveDirection, Orientation, TurtleIndexType},
    Pos3,
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum TurtleUpDown {
    Up,
    Forward,
    Down,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SetupInfoData {
    pub facing: Orientation,
    pub position: Pos3,
    pub index: TurtleIndexType,
    pub world: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum T2SPackets {
    Batch(Vec<T2SPackets>),
    SetupInfo(SetupInfoData),
    Moved {
        direction: MoveDirection,
    },
    SetMaxFuel(i32),
    SetPos(Pos3),
    SetOrientation(Orientation),
    WorldUpdate(String),
    InventoryUpdate(TurtleInventory),
    NameUpdate(String),
    FuelUpdate(i32),
    Blocks {
        up: Maybe<String>,
        down: Maybe<String>,
        front: Maybe<String>,
    },
    Ping,
}
#[derive(serde::Serialize, serde::Deserialize)]
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
    RunLuaCode(String),
    GetSetupInfo,
}
