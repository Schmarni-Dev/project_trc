pub mod chunk;
pub mod components;
pub mod events;
pub mod executable_files;
pub mod idk;
pub mod input;
pub mod inventory;
pub mod lerp_transform;
pub mod primary_turtle_ui;
pub mod primary_ui;
pub mod turtle;
pub mod turtle_input;
pub mod turtle_inventory;
pub mod turtle_movement;
pub mod websocket;
use std::{path::PathBuf, sync::Arc};
pub mod bundels;
pub mod external_inv_support;
pub mod raycast;
pub mod systems;
pub mod turtle_stuff;
pub mod util;
pub mod ws;

pub use actually_usable_voxel_mesh_gen as voxel_meshing;
use bevy::prelude::{Deref, DerefMut, Resource};

#[derive(Resource)]
pub struct WorldState {
    pub curr_world: Option<String>,
    pub worlds: Vec<String>,
}
#[derive(Resource)]
pub struct InputState {
    pub block_camera_updates: bool,
}
#[derive(Resource, DerefMut, Deref)]
pub struct DoBlockRaymarch(pub bool);

#[derive(Resource)]
pub struct BlockBlacklist {
    pub block_render_blacklist: Arc<[String]>,
}
#[derive(Resource)]
pub struct MiscState {
    pub hovered_block: Option<String>,
}
#[derive(Resource)]
pub struct ShowFileDialog {
    pub show: bool,
    pub conntents: String,
    pub file: PathBuf,
}
