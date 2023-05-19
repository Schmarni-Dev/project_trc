use actually_usable_voxel_mesh_gen::*;
use common::Pos3;

struct ChunkFake;

impl data::ChunkData for ChunkFake {
    fn get_chunk_size(&self) -> i32 {
        16
    }
    fn has_neighbour(&self, _pos: &Pos3, _side: &data::Side) -> bool {
        false
    }
    fn get_color_seed_for_block(&self, _pos: &Pos3) -> String {
        "dwasdwasd".to_owned()
    }
    fn does_block_exits(&self, pos: &Pos3) -> bool {
        pos == &Pos3::default()
    }
}

fn main() {
    generate_mesh_for_chunk(&ChunkFake);
}
