use bevy::prelude::*;
use common::turtle::Turtle;
use std::ops::{Deref, DerefMut};

use crate::{
    components::LerpTransform,
    util::{pos3_to_vec3, quat_from_dir},
};

pub const TURTLE_LERP_TIME: f32 = 0.5;

#[derive(Component, Debug)]
pub struct TurtleInstance {
    pub turtle: Turtle,
    /// Remove in future just to lazy rn
    #[allow(dead_code)]
    pub active: bool,
}

impl TurtleInstance {
    pub fn new(chunk_data: Turtle) -> TurtleInstance {
        TurtleInstance {
            turtle: chunk_data,
            active: false,
        }
    }
}

impl Deref for TurtleInstance {
    type Target = Turtle;
    fn deref(&self) -> &Self::Target {
        &self.turtle
    }
}

impl DerefMut for TurtleInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.turtle
    }
}

#[derive(Bundle)]
pub struct TurtleBundle {
    turtle: TurtleInstance,
    scene: Handle<Scene>,
    lerp_pos: LerpTransform,
    transform: TransformBundle,
    visability: VisibilityBundle,
}

#[derive(Event)]
pub struct SpawnTurtle {
    pub turtle: Turtle,
    pub active: bool,
}

#[derive(Resource)]
pub struct TurtleModels {
    pub active_turtle: Handle<Scene>,
    pub inactive_turtle: Handle<Scene>,
}

impl TurtleModels {
    pub fn get_correct_mdl(&self, active: bool) -> Handle<Scene> {
        match active {
            true => self.active_turtle.clone_weak(),
            false => self.inactive_turtle.clone_weak(),
        }
    }
}

pub fn turtle_spawner(
    mut events: EventReader<SpawnTurtle>,
    models: Res<TurtleModels>,
    mut commands: Commands,
) {
    for t in events.read() {
        let end_pos = pos3_to_vec3(t.turtle.position) + Vec3::splat(0.5);
        let mut lerp = LerpTransform::new(
            end_pos - Vec3::NEG_Y,
            quat_from_dir(
                pos3_to_vec3(t.turtle.orientation.get_forward_vec()),
                Vec3::Y,
            ),
        );

        lerp.lerp_pos_to(end_pos, TURTLE_LERP_TIME);

        commands.spawn(TurtleBundle {
            turtle: TurtleInstance::new(t.turtle.clone()),

            scene: models.get_correct_mdl(t.active),
            transform: TransformBundle {
                local: Transform::from_translation(end_pos),
                ..Default::default()
            },

            lerp_pos: lerp,
            visability: Default::default(),
        });
    }
}
