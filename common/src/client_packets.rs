use crate::{
    turtle::{self, Turtle},
    world_data::{Block, World},
    Pos3,
};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, bevy::ecs::event::Event)]
pub enum C2SPackets {
    MoveTurtle {
        index: i32,
        direction: turtle::MoveDirection,
    },
    RequestTurtles(String),
    RequestWorlds,
    RequestWorld(String),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct MovedTurtleData {
    pub index: i32,
    pub new_orientation: turtle::Orientation,
    pub new_pos: Pos3,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SetTurtlesData {
    pub turtles: Vec<Turtle>,
    pub world: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, bevy::ecs::event::Event)]
pub enum S2CPackets {
    MovedTurtle(MovedTurtleData),
    SetTurtles(SetTurtlesData),
    Worlds(Vec<String>),
    WorldUpdate(Block),
    SetWorld(World),
}
