use std::sync::Arc;

use crate::voxel_meshing;
use bevy::prelude::*;
use common::{
    world_data::{get_chunk_relative_pos, Chunk, CHUNK_SIZE},
    Pos3,
};
use voxel_meshing::{data::ChunkData, generate_mesh_for_chunk};

#[derive(Deref, DerefMut)]
pub struct ClientChunk {
    pub blacklist: Arc<[String]>,
    #[deref]
    chunk: Chunk,
}

impl ClientChunk {
    pub fn new(pos: Pos3) -> ClientChunk {
        ClientChunk {
            chunk: Chunk::new(pos),
            blacklist: Arc::new([]),
        }
    }
    pub fn from_chunk(chunk: Chunk) -> ClientChunk {
        ClientChunk {
            chunk,
            blacklist: Arc::new([]),
        }
    }
    fn get_block_if_exists(&self,pos: &Pos3) -> Option<String> {
        if self.chunk.does_block_exist(pos){
            return self.chunk.get_block_id(pos);
        }
        None
    }
}

impl ChunkData for ClientChunk {
    fn does_block_exits(&self, pos: &Pos3) -> bool {
        self.get_block_if_exists(pos).is_some_and(|b| !self.blacklist.contains(&b))
    }
    fn get_chunk_size(&self) -> i32 {
        CHUNK_SIZE
    }
    fn has_neighbour(&self, pos: &Pos3, side: &voxel_meshing::data::Side) -> bool {
        let pos = get_chunk_relative_pos(pos);
        let neighbour_pos = pos + side.side_to_rel_pos();
        let mut oob = neighbour_pos.x < 0 || neighbour_pos.y < 0 || neighbour_pos.z < 0;
        oob |= neighbour_pos.x > CHUNK_SIZE
            || neighbour_pos.y > CHUNK_SIZE
            || neighbour_pos.z > CHUNK_SIZE;
        if oob {
            return false;
        }
        self.does_block_exits(&neighbour_pos)
    }
    fn get_color_seed_for_block(&self, pos: &Pos3) -> String {
        self.get_block_id(pos).unwrap_or_default()
    }
}

pub fn do_mesh_shit(mesh: &mut Mesh, chunk: &ClientChunk) {
    let data = generate_mesh_for_chunk(chunk);
    let mut vertecies: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    data.into_iter().for_each(|bfd| {
        let vertexes = bfd
            .vertecies
            .iter()
            .map(|v| [v.x, v.y, v.z])
            .collect::<Vec<[i32; 3]>>();
        normals.append(
            &mut vertexes
                .iter()
                .map(|_| {
                    let mut color = bfd
                        .color
                        .iter()
                        .map(|c| *c as f32 / 255f32)
                        .collect::<Vec<f32>>();
                    color.push(1.);
                    colors.push(color.try_into().unwrap());
                    // let i = i as u32 * 4u32;
                    // mesh.set_indices(Some(Indices::U32(vec![i, i + 1, i + 2])));
                    // mesh.set_indices(Some(Indices::U32(vec![i + 2, i + 1, i + 3])));
                    let normal = &bfd.normal;
                    [normal.x as f32, normal.y as f32, normal.z as f32]
                })
                .collect::<Vec<[f32; 3]>>(),
        );
        vertecies.append(
            &mut vertexes
                .into_iter()
                .map(|v| [v[0] as f32, v[1] as f32, v[2] as f32])
                .collect::<Vec<[f32; 3]>>(),
        );
    });
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertecies);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
}
