use bevy::prelude::{Deref, DerefMut};

use crate::{
    turtle::{self, TurtleInventory, Turtle},
    turtle_packets::TurtleUpDown,
    world_data::{Block, World},
    Pos3,
};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, bevy::ecs::event::Event)]
pub enum C2SPackets {
    MoveTurtle {
        index: i32,
        world: String,
        direction: turtle::MoveDirection,
    },
    TurtleSelectSlot {
        index: i32,
        world: String,
        slot: u32,
    },
    RequestTurtles(String),
    RequestWorlds,
    RequestWorld(String),
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
    SendLuaToTurtle {
        index: i32,
        world: String,
        code: String,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct MovedTurtleData {
    pub index: i32,
    pub world: String,
    pub new_orientation: turtle::Orientation,
    pub new_pos: Pos3,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct UpdateTurtleData<T> {
    pub index: i32,
    pub world: String,
    pub data: T,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Deref, DerefMut)]
pub struct SetTurtlesData {
    pub turtles: Vec<Turtle>,
    #[deref]
    pub world: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, bevy::ecs::event::Event)]
pub enum S2CPackets {
    MovedTurtle(MovedTurtleData),
    TurtleInventoryUpdate(UpdateTurtleData<TurtleInventory>),
    TurtleFuelUpdate(UpdateTurtleData<i32>),
    SetTurtles(SetTurtlesData),
    Worlds(Vec<String>),
    WorldUpdate(Block),
    SetWorld(World),
}
