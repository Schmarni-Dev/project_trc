use crate::data::Side;
use common::Pos3;

pub fn get_vertecies_from_side(side: &Side) -> [Pos3; 6] {
    let points = match side {
        Side::PosX => [
            Pos3::new(1, 1, 1),
            Pos3::new(1, 0, 1),
            Pos3::new(1, 1, 0),
            Pos3::new(1, 0, 0),
        ],

        Side::NegX => [
            Pos3::new(0, 1, 0),
            Pos3::new(0, 0, 0),
            Pos3::new(0, 1, 1),
            Pos3::new(0, 0, 1),
        ],

        Side::PosY => [
            Pos3::new(0, 1, 1),
            Pos3::new(1, 1, 1),
            Pos3::new(0, 1, 0),
            Pos3::new(1, 1, 0),
        ],

        Side::NegY => [
            Pos3::new(1, 0, 1),
            Pos3::new(0, 0, 1),
            Pos3::new(1, 0, 0),
            Pos3::new(0, 0, 0),
        ],

        Side::PosZ => [
            Pos3::new(1, 0, 0),
            Pos3::new(0, 0, 0),
            Pos3::new(1, 1, 0),
            Pos3::new(0, 1, 0),
        ],

        Side::NegZ => [
            Pos3::new(0, 0, 1),
            Pos3::new(1, 0, 1),
            Pos3::new(0, 1, 1),
            Pos3::new(1, 1, 1),
        ],
    };
    [
        points[0], points[1], points[2], points[2], points[1], points[3],
    ]
}
