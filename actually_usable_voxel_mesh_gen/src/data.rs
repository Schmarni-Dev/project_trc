use common::Pos3;

pub enum Side {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

impl Side {
    pub fn side_to_rel_pos(&self) -> Pos3 {
        match self {
            Side::NegX => Pos3 { x: -1, y: 0, z: 0 },
            Side::PosX => Pos3 { x: 1, y: 0, z: 0 },
            Side::NegY => Pos3 { x: 0, y: -1, z: 0 },
            Side::PosY => Pos3 { x: 0, y: 1, z: 0 },
            Side::NegZ => Pos3 { x: 0, y: 0, z: -1 },
            Side::PosZ => Pos3 { x: 0, y: 0, z: 1 },
        }
    }
}

pub trait ChunkData: std::marker::Send + std::marker::Sync {
    fn has_neighbour(&self, pos: &Pos3, side: &Side) -> bool;
    fn get_chunk_size(&self) -> i32;
    fn get_color_seed_for_block(&self, pos: &Pos3) -> String;
    fn does_block_exits(&self, pos: &Pos3) -> bool;
}

pub struct BlockFaceData {
    pub vertecies: [Pos3; 6],
    pub normal: Pos3,
    pub color: [u8; 3],
}
