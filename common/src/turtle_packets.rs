use crate::{
    turtle::{Inventory, Maybe, MoveDirection, Orientation, TurtleIndexType},
    Pos3,
};

// #[derive(serde::Serialize, serde::Deserialize, Debug)]

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
    InventoryUpdate(Inventory),
    NameUpdate(String),
    FuelUpdate(i32),
    Blocks {
        up: Maybe<String>,
        down: Maybe<String>,
        front: Maybe<String>,
    },
}
#[derive(serde::Serialize, serde::Deserialize)]
pub enum S2TPackets {
    Move(Vec<MoveDirection>),
    GetSetupInfo,
}
