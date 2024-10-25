pub mod extensions;
mod pos3;
pub mod remote_control_packets;
pub mod turtle;
pub mod util;
use bevy_ecs::component::Component;
pub use pos3::Pos3;
pub mod client_packets;
pub mod turtle_packets;
pub mod vec3d;
pub mod world_data;

use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Display, Component, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComputerType {
    Computer,
    Turtle,
    PocketComputer,
}

/// Runtime device id, will change on Device reconnect
#[derive(Clone, Copy, Debug, Display, Component, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(u64);

impl DeviceId {
    /// Creates a new [`DeviceId`].
    ///
    /// # Safety
    /// only has runtime meaning, only get it from the server
    pub unsafe fn new(id: u64) -> DeviceId {
        DeviceId(id)
    }
    pub fn as_raw(&self) -> u64 {
        self.0
    }
}
