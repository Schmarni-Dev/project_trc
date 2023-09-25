use bevy::prelude::*;
use common::world_data::{get_chunk_containing_block, get_chunk_relative_pos};

use crate::{
    components::ChunkInstance, util::vec3_to_pos3, BlockBlacklist, DoBlockRaymarch, InputState,
    MiscState,
};

pub struct RaycastPlugin;

impl Plugin for RaycastPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, block_raycast.run_if(do_block_raycast));
    }
}

const POINTS: u32 = 100;

fn do_block_raycast(block_march: Res<DoBlockRaymarch>) -> bool {
    **block_march
}

fn block_raycast(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    chunk_q: Query<&ChunkInstance>,
    mut misc_state: ResMut<MiscState>,
    block_blacklist: ResMut<BlockBlacklist>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = camera_q.single();

    let block = windows.iter().find_map(|window| {
        let block = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| {
                (0..POINTS).find_map(|distance| {
                    let pos = vec3_to_pos3(ray.get_point(distance as f32).floor());
                    let c_pos = get_chunk_containing_block(&pos);
                    let chunk = chunk_q.iter().find(|chunk| chunk.get_chunk_pos() == &c_pos);
                    let block = chunk.and_then(|c| {
                        let rel = get_chunk_relative_pos(&pos);
                        let b = c.get_block_id(&rel)?;
                        let e = c.does_block_exist(&rel)
                            && !block_blacklist.block_render_blacklist.contains(&b);
                        match e {
                            true => Some(b),
                            false => None,
                        }
                    });
                    block
                })
            });
        block
    });
    misc_state.hovered_block = block.flatten();
    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
}
