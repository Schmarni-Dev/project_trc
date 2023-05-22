use bevy::prelude::*;
use common::turtle::Turtle;
use std::ops::{Deref, DerefMut};

use crate::{components::LerpPos, util::pos3_to_vec3};

#[derive(Component)]
pub struct TurtleInstance {
    turtle: Turtle,
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
    lerp_pos: LerpPos,
    transform: TransformBundle,
    visability: VisibilityBundle,
}

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
    for t in events.iter() {
        let end_pos = pos3_to_vec3(t.turtle.position) + Vec3::splat(0.5);
        commands.spawn(TurtleBundle {
            turtle: TurtleInstance::new(t.turtle.clone()),

            scene: models.get_correct_mdl(t.active),
            transform: TransformBundle {
                local: Transform::from_translation(end_pos),
                ..Default::default()
            },

            lerp_pos: LerpPos::new(end_pos, end_pos, 0.5),
            visability: Default::default(),
        });
    }
}
