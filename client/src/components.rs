use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use common::Pos3;

use crate::idk::ClientChunk;

#[derive(Component)]
pub struct LerpTransform {
    pub pos_time: f32,
    pub start_pos: Vec3,
    pub end_pos: Vec3,
    pub current_pos_time: f32,
    pub start_rot: Quat,
    pub end_rot: Quat,
    pub current_rot_time: f32,
    pub rot_time: f32,
}

impl LerpTransform {
    pub fn new(pos: Vec3, rot: Quat) -> LerpTransform {
        LerpTransform {
            pos_time: 1.,
            start_pos: pos,
            end_pos: pos,
            current_pos_time: 1.,
            start_rot: rot,
            end_rot: rot,
            current_rot_time: 1.,
            rot_time: 1.,
        }
    }
    pub fn lerp_rot_to(&mut self, end_rot: Quat, time: f32) -> &mut Self {
        self.start_rot = self.end_rot;
        self.end_rot = end_rot;
        self.current_rot_time = 0.;
        self.rot_time = time;
        self
    }
    pub fn lerp_pos_to(&mut self, end_pos: Vec3, time: f32) -> &mut Self {
        self.start_pos = self.end_pos;
        self.end_pos = end_pos;
        self.current_pos_time = 0.;
        self.pos_time = time;
        self
    }
}

#[derive(Component)]
pub struct ChunkInstance {
    chunk_pos: Pos3,
    chunk_data: ClientChunk,
}

impl ChunkInstance {
    pub fn new(chunk_pos: Pos3, chunk_data: ClientChunk) -> ChunkInstance {
        ChunkInstance {
            chunk_pos,
            chunk_data,
        }
    }
    pub fn get_chunk_pos(&self) -> &Pos3 {
        &self.chunk_pos
    }
    pub fn inner_mut(&mut self) -> &mut ClientChunk {
        &mut self.chunk_data
    }
}

impl Deref for ChunkInstance {
    type Target = ClientChunk;
    fn deref(&self) -> &Self::Target {
        &self.chunk_data
    }
}

// impl DerefMut for ChunkInstance {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.chunk_data
//     }
// }
