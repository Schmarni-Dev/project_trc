use bevy::prelude::{Deref, DerefMut};

use crate::{
    turtle::{self, Turtle, TurtleInventory},
    world_data::{Block, World},
    Pos3,
};

// Needed: start executable on turtle
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, bevy::ecs::event::Event)]
pub enum C2SPackets {
    RequestTurtles(String),
    RequestWorlds,
    RequestWorld(String),
    SendLuaToTurtle {
        index: i32,
        world: String,
        code: String,
    },
    StdInForTurtle {
        index: i32,
        value: String,
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

// Needed: turtle requesting input from client(might need to somehow sync that? or just first come
// first serve)
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, bevy::ecs::event::Event)]
pub enum S2CPackets {
    MovedTurtle(MovedTurtleData),
    TurtleInventoryUpdate(UpdateTurtleData<Box<TurtleInventory>>),
    TurtleFuelUpdate(UpdateTurtleData<i32>),
    SetTurtles(SetTurtlesData),
    Worlds(Vec<String>),
    WorldUpdate(Block),
    SetWorld(World),
    StdOutFromTurtle { index: i32, value: String },
}
