use std::fmt::Debug;

use bevy::prelude::{Vec3, Quat, Mat3};
use common::Pos3;

#[inline]
/// Copied from bevy and modified
pub fn quat_from_dir(direction: Vec3, up: Vec3) -> Quat {
    let forward = -direction.normalize();
    let right = up.cross(forward).normalize();
    let up = forward.cross(right);
    Quat::from_mat3(&Mat3::from_cols(right, up, forward))
}
#[allow(dead_code)]
pub fn debug_println(val: impl Debug) {
    println!("{:?}", val)
}

#[allow(dead_code)]
pub fn pos3_to_vec3(val: Pos3) -> Vec3 {
    Vec3::new(val.x as f32, val.y as f32, val.z as f32)
}

#[allow(dead_code)]
pub fn vec3_to_pos3(val: Vec3) -> Pos3 {
    Pos3::new(val.x as i32, val.y as i32, val.z as i32)
}
