use bevy::prelude::*;
use common::{turtle::Orientation, Pos3};

use crate::lerp_transform::LerpTransform;
// Might pull this somewhere else and make it a generic computer index
#[derive(Clone, Copy, Deref, DerefMut, Component, Debug)]
pub struct TurtleIndex(pub i32);
// Might pull this somewhere else and make it a generic POI Pos
#[derive(Clone, Copy, Deref, DerefMut, Component, Debug)]
pub struct TurtlePosition(pub Pos3);
// Might pull this somewhere else and make it a generic POI Dir
#[derive(Clone, Copy, Deref, DerefMut, Component, Debug)]
pub struct TurtleDirection(pub Orientation);
#[derive(Clone, Copy, Deref, DerefMut, Component, Debug)]
pub struct TurtleFuel(pub i32);
#[derive(Clone, Copy, Deref, DerefMut, Component, Debug)]
pub struct TurtleMaxFuel(pub i32);
// Might pull this somewhere else and make it a generic Online indicator
#[derive(Clone, Copy, Deref, DerefMut, Component, Debug)]
pub struct TurtleIsOnline(pub bool);
// Might pull this somewhere else and make it a generic World indicator
#[derive(Clone, Deref, DerefMut, Component, Debug)]
pub struct TurtleWorld(pub String);

#[derive(Clone, Copy, Component, Debug)]
pub struct Turtle;

#[derive(Clone, Copy, Component, Debug)]
pub struct LocallyControlledTurtle;
#[derive(Clone, Copy, Component, Debug)]
pub struct RemotelyControlledTurtle;

#[derive(Bundle)]
pub struct TurtleBundle {
    pub index: TurtleIndex,
    pub name: Name,
    pub pos: TurtlePosition,
    pub dir: TurtleDirection,
    pub fuel: TurtleFuel,
    pub max_fuel: TurtleMaxFuel,
    pub online: TurtleIsOnline,
    pub world: TurtleWorld,
    pub model: SceneBundle,
    pub lerp: LerpTransform,
    pub tag: Turtle,
}

impl TurtleBundle {
    pub fn from_common_turtle(turtle: common::turtle::Turtle, models: &TurtleModels) -> Self {
        Self {
            index: TurtleIndex(turtle.index),
            name: Name::new(turtle.name),
            pos: TurtlePosition(turtle.position),
            dir: TurtleDirection(turtle.orientation),
            fuel: TurtleFuel(turtle.fuel),
            max_fuel: TurtleMaxFuel(turtle.max_fuel),
            online: TurtleIsOnline(turtle.is_online),
            world: TurtleWorld(turtle.world),
            model: SceneBundle {
                scene: models.online.clone_weak(),
                transform: Transform::IDENTITY,
                ..default()
            },
            lerp: LerpTransform::new(Vec3::ZERO, Quat::IDENTITY),
            tag: Turtle,
        }
    }
}

#[derive(Resource)]
pub struct PrimaryTurtle(pub Entity);
impl Default for PrimaryTurtle {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

pub struct TurtlePlugin;
impl Plugin for TurtlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PrimaryTurtle>();
        app.add_systems(Startup, setup_turtle_models);
    }
}

#[derive(Resource)]
pub struct TurtleModels {
    pub local_control: Handle<Scene>,
    pub remote_control: Handle<Scene>,
    pub online: Handle<Scene>,
    pub offline: Handle<Scene>,
}

fn setup_turtle_models(asset_server: Res<AssetServer>, mut cmds: Commands) {
    cmds.insert_resource(TurtleModels {
        local_control: asset_server.load("turtle.gltf#Scene0"),
        remote_control: asset_server.load("turtle.gltf#Scene0"),
        online: asset_server.load("turtle_inactive.gltf#Scene0"),
        offline: asset_server.load("turtle_inactive.gltf#Scene0"),
    })
}
