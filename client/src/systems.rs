use bevy::prelude::*;

use crate::components::LerpPos;
pub struct Systems;

impl Plugin for Systems {
    fn build(&self, app: &mut App) {
        // add things to your app here
        app.add_system(lerp_pos_system);
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

pub fn move_turtle(mut query: Query<(&mut Transform, &mut LerpPos)>) {}
