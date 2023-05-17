use crate::turtle;
use crate::Pos3;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum C2STrutlePacket {
    MoveTurtle {
        index: i32,
        direction: turtle::MoveDirection,
    },
    RequestTurtles,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum S2CTrutlePacket {
    MovedTurtle {
        index: i32,
        new_orientation: turtle::Orientation,
        new_pos: Pos3,
    },
    RequestedTurtles(Vec<turtle::Turtle>),
}
