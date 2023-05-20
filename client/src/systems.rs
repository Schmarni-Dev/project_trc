use bevy::prelude::*;
use common::client_packets::S2CPackets;

use crate::{components::LerpPos, turtle_stuff::TurtleInstance, util::pos3_to_vec3};
pub struct Systems;

impl Plugin for Systems {
    fn build(&self, app: &mut App) {
        // add things to your app here
        app.add_system(lerp_pos_system);
        // app.add_system(move_turtle);
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
