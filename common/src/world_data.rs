use crate::Pos3;
use serde::{Deserialize, Serialize};

use super::vec3d::Vec3D;

const CHUNK_SIZE: i16 = 16;

pub fn get_chunk_containing_block(pos: &Pos3) -> Pos3 {
    Pos3::new(
        ((pos.x as f32) / (CHUNK_SIZE as f32)).floor() as i16,
        ((pos.y as f32) / (CHUNK_SIZE as f32)).floor() as i16,
        ((pos.z as f32) / (CHUNK_SIZE as f32)).floor() as i16,
    )
}

pub fn get_chunk_relative_pos(pos: &Pos3) -> Pos3 {
    Pos3::new(pos.x % CHUNK_SIZE, pos.y % CHUNK_SIZE, pos.z % CHUNK_SIZE)
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct World {
    chunks: Vec3D<Chunk>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct Chunk {
    /// uses Global Block Pos Not Relative Pos
    blocks: Vec3D<Block>,
    pos: Pos3,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct Block {
    id: String,
    /// Global pos
    pos: Pos3,
}

impl World {
    pub fn get_block(&self, pos: &Pos3) -> Option<&Block> {
        self.chunks
            .get(&get_chunk_containing_block(pos))?
            .blocks
            .get(pos)
    }
}
