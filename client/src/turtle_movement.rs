use bevy::prelude::*;
use common::client_packets::{C2SPacket, S2CPacket};

use crate::{
    lerp_transform::LerpTransform, turtle::{Turtle, TurtleDirection, TurtleIndex, TurtlePosition, TurtleWorld}, turtle_stuff::TURTLE_LERP_TIME, util::{pos3_to_vec3, quat_from_dir}, websocket::{WebSocketPlugin, WsMsgRecived}
};
pub struct TurtleMovementPlugin;
impl Plugin for TurtleMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_position_updates);
        app.add_systems(Update, handle_orientation_updates);
        if app.is_plugin_added::<WebSocketPlugin<C2SPacket, S2CPacket>>() {
            app.add_systems(Update, handle_mov_packet);
        } else {
            warn!("Websocket Plugin for Turtles not added, networking will be broken!");
        }
    }
}
fn handle_orientation_updates(
    mut query: Query<(&TurtleDirection, &mut LerpTransform), Changed<TurtleDirection>>,
) {
    for (dir, mut lerp) in &mut query {
        lerp.lerp_rot_to(
            quat_from_dir(pos3_to_vec3(dir.get_forward_vec()), Vec3::Y),
            TURTLE_LERP_TIME,
        );
    }
}
fn handle_position_updates(
    mut query: Query<(&TurtlePosition, &mut LerpTransform), Changed<TurtlePosition>>,
) {
    for (pos, mut lerp) in &mut query {
        lerp.lerp_pos_to(pos3_to_vec3(**pos) + Vec3::splat(0.5), TURTLE_LERP_TIME);
    }
}

fn handle_mov_packet(
    mut events: EventReader<WsMsgRecived<S2CPacket>>,
    mut query: Query<
        (
            &mut TurtlePosition,
            &mut TurtleDirection,
            &TurtleIndex,
            &TurtleWorld,
        ),
        With<Turtle>,
    >,
) {
    while let Some(WsMsgRecived(S2CPacket::MovedTurtle(e))) = events.read().next() {
        for (mut pos, mut dir, _, _) in query
            .iter_mut()
            .filter(|(_, _, id, world)| id.0 == e.index && world.0 == e.world)
        {
            pos.0 = e.new_pos;
            dir.0 = e.new_orientation;
        }
    }
}
