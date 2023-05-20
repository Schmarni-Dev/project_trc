use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use common::Pos3;

use crate::idk::ClientChunk;

#[derive(Component)]
pub struct LerpPos {
    pub time: f32,
    pub start_pos: Vec3,
    pub end_pos: Vec3,
    pub current_time: f32,
}

impl LerpPos {
    pub fn new(start_pos: Vec3, end_pos: Vec3, time: f32) -> LerpPos {
        LerpPos {
            time,
            start_pos,
            end_pos,
            current_time: 0.,
        }
    }
    pub fn lerp_to(&mut self, end_pos: Vec3) {
        self.start_pos = self.end_pos;
        self.end_pos = end_pos;
        self.current_time = 0.;
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
}

impl Deref for ChunkInstance {
    type Target = ClientChunk;
    fn deref(&self) -> &Self::Target {
        &self.chunk_data
    }
}

impl DerefMut for ChunkInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.chunk_data
    }
}
