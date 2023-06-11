use bevy::{math::vec3, prelude::*};
use common::world_data::CHUNK_SIZE;

use crate::{
    components::{ChunkInstance, LerpTransform},
    idk::{do_mesh_shit, ClientChunk},
    util::pos3_to_vec3,
};

#[derive(Bundle)]
pub struct ChunkBundle {
    chunk: ChunkInstance,
    pbr_bundle: PbrBundle,
    lerp_comp: LerpTransform,
}

impl ChunkBundle {
    pub fn new(
        chunk: ClientChunk,
        meshes: &mut ResMut<Assets<Mesh>>,
        material: Handle<StandardMaterial>,
    ) -> ChunkBundle {
        info!("HELLO CHUNK!");
        let pos = chunk.get_chunk_pos();
        let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
        do_mesh_shit(&mut mesh, &chunk);
        let end_pos = pos3_to_vec3(pos.scale(CHUNK_SIZE));
        let lerp_pos = end_pos - vec3(0., CHUNK_SIZE as f32, 0.);
        info!("pos: {:#?}", lerp_pos);
        let mut lerp_comp = LerpTransform::new(lerp_pos, Quat::IDENTITY);
        lerp_comp.lerp_pos_to(end_pos, 0.5);
        ChunkBundle {
            chunk: ChunkInstance::new(pos, chunk),
            pbr_bundle: PbrBundle {
                mesh: meshes.add(mesh),
                material,
                transform: Transform::from_translation(end_pos),
                ..Default::default()
            },
            lerp_comp,
        }
    }
}
