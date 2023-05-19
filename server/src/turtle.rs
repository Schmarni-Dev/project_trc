use common::Pos3;

use crate::TurtleMap;
#[allow(dead_code)]
pub fn non_mc_move(packet: common::C2SPackets, turtles: TurtleMap) -> Option<common::S2CPackets> {
    match packet {
        common::C2SPackets::MoveTurtle { index, direction } => {
            if let Some(turtle) = turtles.lock().unwrap().get_mut(&index) {
                turtle.get_forward_vec();
                match direction {
                    common::turtle::MoveDirection::Forward => {
                        turtle.position += turtle.get_forward_vec()
                    }
                    common::turtle::MoveDirection::Back => {
                        turtle.position += turtle.get_forward_vec().scale(-1)
                    }
                    common::turtle::MoveDirection::Up => turtle.position += Pos3::new(0, 1, 0),
                    common::turtle::MoveDirection::Down => turtle.position += Pos3::new(0, -1, 0),
                    common::turtle::MoveDirection::Left => {
                        turtle.turn(common::turtle::TurnDir::Left);
                    }
                    common::turtle::MoveDirection::Right => {
                        turtle.turn(common::turtle::TurnDir::Right);
                    }
                }
                return Some(common::S2CPackets::MovedTurtle {
                    index,
                    new_orientation: turtle.orientation,
                    new_pos: turtle.position,
                });
            };
            None
        }
        _ => {
            panic!("WRONG PACKET IN non_mc_move");
        }
    }
}
