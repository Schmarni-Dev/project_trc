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
    Pos3::new(
        (pos.x as u32 % CHUNK_SIZE as u32) as i32,
        (pos.y as u32 % CHUNK_SIZE as u32) as i32,
        (pos.z as u32 % CHUNK_SIZE as u32) as i32,
    )
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct World {
    name: String,
    chunks: Vec3D<Chunk>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Chunk {
    /// Blocks Are Stored in Chunk Local Positions
    blocks: Vec3D<Block>,
    /// the Chunk Positions. Not in Meters!
    pos: Pos3,
}

impl Chunk {
    pub fn does_block_exist(&self, pos: &Pos3) -> bool {
        let out = self.blocks.get(pos);
        out.is_some_and(|b| !b.is_air)
    }
    pub fn new(pos: Pos3) -> Chunk {
        Chunk {
            blocks: Vec3D::new(),
            pos,
        }
    }
    // pub fn add_block(&mut self, pos: Pos3, name: &str) {
    //     self.blocks.insert(
    //         pos,
    //         Block {
    //             id: name.to_owned(),
    //             pos: pos + self.pos.scale(CHUNK_SIZE),
    //         },
    //     )
    // }
    pub fn set_block(&mut self, block: Block) {
        self.blocks
            .insert(get_chunk_relative_pos(&block.pos), block);
    }
    pub fn get_block_id(&self, pos: &Pos3) -> Option<String> {
        self.blocks.get(pos).map(|block| block.id.clone())
    }
    pub fn get_chunk_pos(&self) -> Pos3 {
        self.pos
    }
    pub fn all_blocks(&self) -> Vec3D<Block> {
        self.blocks.clone()
    }
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub world: String,
    pub id: String,
    /// Global pos
    pub pos: Pos3,
    pub is_air: bool,
}

impl Block {
    pub fn new(ident: Option<String>, pos: &Pos3, world_name: &str) -> Block {
        Block {
            is_air: ident.is_none(),
            id: ident.unwrap_or_default(),
            pos: *pos,
            world: world_name.to_owned(),
        }
    }
    pub fn get_pos(&self) -> &Pos3 {
        &self.pos
    }
    pub fn get_name(&self) -> &str {
        &self.id
    }
}

impl World {
    pub fn get_block(&self, pos: &Pos3) -> Option<&Block> {
        self.chunks
            .get(&get_chunk_containing_block(pos))?
            .blocks
            .get(pos)
    }

    pub fn set_block(&mut self, block: Block) {
        let chunk_pos = get_chunk_containing_block(&block.pos);
        self.chunks
            .entry(chunk_pos)
            .or_insert(Chunk::new(chunk_pos))
            .set_block(block);
    }

    pub fn get_chunks(&self) -> &Vec3D<Chunk> {
        &self.chunks
    }

    pub fn new(name: &str) -> World {
        World {
            name: name.to_owned(),
            chunks: Vec3D::new(),
        }
    }
}
