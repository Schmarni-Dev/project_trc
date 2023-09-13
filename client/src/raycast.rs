use bevy::{prelude::*, render::camera::RenderTarget};
use common::world_data::get_chunk_containing_block;

use crate::{util::{pos3_to_vec3, vec3_to_pos3}, components::ChunkInstance};

pub struct RaycastPlugin;

impl Plugin for RaycastPlugin {
    fn build(&self, app: &mut App) {}
}

fn my_cursor_system(
    // need to get window dimensions
    windows: Query<&Window>,
    // query to get camera transform
    camera_q: Query<(&Camera, &GlobalTransform)>,
    chunk_q: Query<&ChunkInstance>
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = camera_q.single();

    for window in windows.iter() {
        window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| {
                for distance in 0..100 {
                    let pos = vec3_to_pos3(ray.get_point(distance as f32).floor());
        let c_pos = get_chunk_containing_block(&pos);
                    let chunk = chunk_q.iter().find(|chunk| chunk.get_chunk_pos() == &c_pos);

                }
            });
    }
    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
}
