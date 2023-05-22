use bevy::{prelude::*, scene::SceneInstance};
use common::client_packets::S2CPackets;
use smooth_bevy_cameras::LookTransform;

use crate::{
    components::LerpPos,
    events::{ActiveTurtleChanged, ActiveTurtleRes},
    turtle_stuff::{TurtleInstance, TurtleModels},
    util::pos3_to_vec3,
};

pub struct Systems;

impl Plugin for Systems {
    fn build(&self, app: &mut App) {
        // add things to your app here
        app.add_system(lerp_pos_system);
        app.add_system(move_turtle);
        app.add_system(update_turtle_model);
        app.add_system(update_cam_point_on_turtle_move);
    }
}

pub fn update_cam_point_on_turtle_move(
    turtles: Query<(&Transform, &LerpPos, &TurtleInstance)>,
    mut cams: Query<&mut LookTransform>,
    active_turtle_res: Res<ActiveTurtleRes>,
) {
    for mut cam in cams.iter_mut() {
        if let Some(t) = turtles
            .iter()
            .find(|(_, _, t)| t.index == active_turtle_res.0)
        {
            if t.1.current_time == 1f32 {
                return;
            }
            cam.eye = (cam.eye - cam.target) + t.0.translation;
            cam.target = t.0.translation;
        }
    }
}

pub fn lerp_pos_system(time: Res<Time>, mut query: Query<(&mut Transform, &mut LerpPos)>) {
    for (mut transform, mut lerp_pos) in &mut query {
        // lerp_pos.start_pos
        lerp_pos.current_time =
            (lerp_pos.current_time + (time.delta_seconds() / lerp_pos.time)).clamp(0., 1.);
        transform.translation = lerp_pos
            .start_pos
            .lerp(lerp_pos.end_pos, lerp_pos.current_time);
    }
}

pub fn update_turtle_model(
    mut commands: Commands,
    models: Res<TurtleModels>,
    mut query: Query<(Entity, &TurtleInstance)>,
    mut event: EventReader<ActiveTurtleChanged>,
) {
    for e in event.iter() {
        query.iter_mut().for_each(|(entity, t)| {
            commands
                .entity(entity)
                .remove::<Handle<Scene>>()
                .insert(models.get_correct_mdl(t.index == e.0));
        });
    }
}

pub fn move_turtle(
    mut query: Query<(&TurtleInstance, &mut LerpPos)>,
    mut event: EventReader<S2CPackets>,
) {
    // TODO: Add Orientation
    while let Some(S2CPackets::MovedTurtle(e)) = event.iter().next() {
        query
            .iter_mut()
            .filter(|(t, _)| t.index == e.index)
            .for_each(|(_, mut lerp)| lerp.lerp_to(pos3_to_vec3(e.new_pos) + Vec3::splat(0.5)));
    }
}
