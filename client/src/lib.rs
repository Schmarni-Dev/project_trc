pub mod components;
pub mod events;
pub mod idk;
pub mod input;
use std::{sync::Arc, path::PathBuf};

pub use actually_usable_voxel_mesh_gen as voxel_meshing;
use bevy::prelude::{Deref, DerefMut, Resource};
pub mod bundels;
pub mod raycast;
pub mod systems;
pub mod turtle_stuff;
pub mod util;
pub mod ws;

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
