use crate::Pos3;
use serde::{Deserialize, Serialize};

use super::vec3d::Vec3D;

pub const CHUNK_SIZE: i32 = 16;

pub fn get_chunk_containing_block(pos: &Pos3) -> Pos3 {
    Pos3::new(
        ((pos.x as f32) / (CHUNK_SIZE as f32)).floor() as i32,
        ((pos.y as f32) / (CHUNK_SIZE as f32)).floor() as i32,
        ((pos.z as f32) / (CHUNK_SIZE as f32)).floor() as i32,
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
    /// Blocks Are Stored in Chunk Local Positions
    blocks: Vec3D<Block>,
    /// the Chunk Positions. Not in Meters!
    pos: Pos3,
}

impl Chunk {
    pub fn does_block_exist(&self, pos: &Pos3) -> bool {
        self.blocks.get(pos).is_some()
    }
    pub fn new(pos: Pos3) -> Chunk {
        Chunk {
            blocks: Vec3D::new(),
            pos,
        }
    }
    pub fn add_block(&mut self, pos: Pos3, name: &str) {
        self.blocks.insert(
            pos,
            Block {
                id: name.to_owned(),
                pos: pos + self.pos.scale(CHUNK_SIZE),
            },
        )
    }
    pub fn get_block_id(&self, pos: &Pos3) -> Option<String> {
        self.blocks.get(pos).map(|block| block.id.clone())
    }
    pub fn get_chunk_pos(&self) -> Pos3 {
        self.pos
    }
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
