use crate::{turtle, Pos3};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum C2SPackets {
    MoveTurtle {
        index: i32,
        direction: turtle::MoveDirection,
    },
    RequestTurtles,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct MovedTurtleData {
    pub index: i32,
    pub new_orientation: turtle::Orientation,
    pub new_pos: Pos3,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum S2CPackets {
    MovedTurtle(MovedTurtleData),
    RequestedTurtles(Vec<turtle::Turtle>),
    TurtleConnected(turtle::Turtle),
}
