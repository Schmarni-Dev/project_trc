use std::fmt::Debug;

use bevy::prelude::Vec3;
use common::Pos3;
#[allow(dead_code)]
pub fn debug_println(val: impl Debug) {
    println!("{:?}", val)
}

#[allow(dead_code)]
pub fn pos3_to_vec3(val: Pos3) -> Vec3 {
    Vec3::new(val.x as f32, val.y as f32, val.z as f32)
}
