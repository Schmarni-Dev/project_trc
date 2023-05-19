use bevy::{math::vec3, prelude::*};
use common::world_data::CHUNK_SIZE;

use crate::{
    components::{ChunkInstance, LerpPos},
    idk::{do_mesh_shit, ClientChunk},
    util::pos3_to_vec3,
};

#[derive(Bundle)]
pub struct ChunkBundle {
    chunk: ChunkInstance,
    pbr_bundle: PbrBundle,
    lerp_pos: LerpPos,
}

impl ChunkBundle {
    pub fn new(
        chunk: ClientChunk,
        meshes: &mut ResMut<Assets<Mesh>>,
        material: Handle<StandardMaterial>,
    ) -> ChunkBundle {
        let pos = chunk.get_chunk_pos();
        let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
        do_mesh_shit(&mut mesh, &chunk);
        let end_pos = pos3_to_vec3(pos.scale(CHUNK_SIZE));
        ChunkBundle {
            chunk: ChunkInstance::new(pos, chunk),
            pbr_bundle: PbrBundle {
                mesh: meshes.add(mesh),
                material,
                transform: Transform::from_translation(end_pos),
                ..Default::default()
            },
            lerp_pos: LerpPos::new(end_pos - vec3(0., CHUNK_SIZE as f32, 0.), end_pos, 0.5),
        }
    }
}
