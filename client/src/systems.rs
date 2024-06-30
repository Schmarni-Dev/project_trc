use bevy::{prelude::*, render::render_asset::RenderAssetUsages};
use common::client_packets::S2CPacket;
use smooth_bevy_cameras::LookTransform;

use crate::{
    events::{ActiveTurtleChanged, ActiveTurtleRes},
    lerp_transform::LerpTransform,
    turtle_stuff::{TurtleInstance, TurtleModels, TURTLE_LERP_TIME},
    util::{pos3_to_vec3, quat_from_dir},
    BlockBlacklist,
};

#[derive(Clone, Copy, Component, Reflect)]
pub struct ActiveTurtle;

pub struct Systems;

impl Plugin for Systems {
    fn build(&self, app: &mut App) {
        // add things to your app here
        app.add_systems(Update, move_turtle);
        app.add_systems(Update, update_turtle_model);
        app.add_systems(Update, update_turtle_component);
        app.add_systems(Update, update_cam_point_on_turtle_move);
        app.add_systems(Update, update_cam_point_on_turtle_select);
    }
}

fn update_turtle_component(
    turtles: Query<(Entity, &TurtleInstance, Has<ActiveTurtle>)>,
    mut cmds: Commands,
    mut event: EventReader<ActiveTurtleChanged>,
) {
    for active in event.read() {
        for (e, t, a) in &turtles {
            match (a, t.index == active.0) {
                (false, true) => {
                    cmds.entity(e).insert(ActiveTurtle);
                }
                (true, false) => {
                    cmds.entity(e).remove::<ActiveTurtle>();
                }
                _ => {}
            }
        }
    }
}

pub fn update_cam_point_on_turtle_select(
    turtles: Query<(&Transform, &LerpTransform, &TurtleInstance)>,
    mut cams: Query<&mut LookTransform>,
    mut event: EventReader<ActiveTurtleChanged>,
) {
    for e in event.read() {
        for mut cam in cams.iter_mut() {
            if let Some(t) = turtles.iter().find(|(_, _, t)| t.index == e.0) {
                cam.eye = (cam.eye - cam.target) + t.0.translation;
                cam.target = t.0.translation;
            }
        }
    }
}

pub fn update_cam_point_on_turtle_move(
    turtles: Query<(&Transform, &LerpTransform, &TurtleInstance)>,
    mut cams: Query<&mut LookTransform>,
    active_turtle_res: Res<ActiveTurtleRes>,
) {
    for mut cam in cams.iter_mut() {
        if let Some(t) = turtles
            .iter()
            .find(|(_, _, t)| t.index == active_turtle_res.0)
        {
            if t.1.current_pos_time == 1f32 {
                return;
            }
            cam.eye = (cam.eye - cam.target) + t.0.translation;
            cam.target = t.0.translation;
        }
    }
}

pub fn update_turtle_model(
    mut commands: Commands,
    models: Res<TurtleModels>,
    mut query: Query<(Entity, &TurtleInstance)>,
    mut event: EventReader<ActiveTurtleChanged>,
) {
    for e in event.read() {
        query.iter_mut().for_each(|(entity, t)| {
            commands
                .entity(entity)
                .remove::<Handle<Scene>>()
                .insert(models.get_correct_mdl(t.index == e.0 && t.is_online));
        });
    }
}

pub fn move_turtle(
    mut query: Query<(&mut TurtleInstance, &mut LerpTransform)>,
    mut event: EventReader<S2CPacket>,
) {
    for msg in event.read() {
        if let S2CPacket::MovedTurtle(e) = msg {
            query
                .iter_mut()
                .filter(|(t, _)| t.index == e.index)
                .for_each(|(mut t, mut lerp)| {
                    lerp.lerp_pos_to(pos3_to_vec3(e.new_pos) + Vec3::splat(0.5), TURTLE_LERP_TIME);
                    t.position = e.new_pos;
                    // let w = quat_from_dir(pos3_to_vec3(t.orientation.get_forward_vec()), Vec3::Y);
                    let r =
                        quat_from_dir(pos3_to_vec3(e.new_orientation.get_forward_vec()), Vec3::Y);
                    t.orientation = e.new_orientation;
                    lerp.lerp_rot_to(r, TURTLE_LERP_TIME);
                });
        }
    }
}
