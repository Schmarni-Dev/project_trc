use crate::{
    extensions, turtle::{Maybe, MoveDirection, Orientation, TurtleIndexType, TurtleInventory}, Pos3
};


#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SetupInfoData {
    pub facing: Orientation,
    pub position: Pos3,
    pub index: TurtleIndexType,
    pub world: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum T2SPackets {
    // Batch(Vec<T2SPackets>),
    SetupInfo(SetupInfoData),
    Moved {
        direction: MoveDirection,
    },
    SetMaxFuel(i32),
    SetPos(Pos3),
    SetOrientation(Orientation),
    ChangeWorld(String),
    InventoryUpdate(Box<TurtleInventory>),
    SetName(String),
    FuelUpdate(i32),
    Blocks {
        up: Maybe<String>,
        down: Maybe<String>,
        front: Maybe<String>,
    },
    Executables(Vec<String>),
    Ping,
    StdOut(String),
    ExtPacket(extensions::T2SPacketExtensions)
}
#[derive(serde::Serialize, serde::Deserialize)]
pub enum S2TPacket {
    RunLuaCode(String),
    GetSetupInfo,
    GetExecutables,
    StdIn(String),
    ExtPacket(extensions::S2TPacketExtensions)
}
