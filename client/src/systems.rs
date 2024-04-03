use bevy::{prelude::*, render::render_asset::RenderAssetUsages};
use common::client_packets::S2CPackets;
use smooth_bevy_cameras::LookTransform;

use crate::{
    components::{ChunkInstance, LerpTransform},
    events::{ActiveTurtleChanged, ActiveTurtleRes},
    idk::do_mesh_shit,
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
        app.add_systems(Update, lerp_pos_system);
        app.add_systems(Update, move_turtle);
        app.add_systems(Update, update_turtle_model);
        app.add_systems(Update, update_turtle_component);
        app.add_systems(Update, update_cam_point_on_turtle_move);
        app.add_systems(Update, update_cam_point_on_turtle_select);
        app.add_systems(Update, lerp_rot_system);
        app.add_systems(Update, chunk_update_mesh);
        app.add_systems(Update, update_chunk_block_blacklists);
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

fn update_chunk_block_blacklists(
    block_blacklist: Res<BlockBlacklist>,
    mut chunks: Query<&mut ChunkInstance>,
) {
    let c = block_blacklist.is_changed();
    chunks
        .iter_mut()
        .filter(|ch| ch.setup | c)
        .for_each(|mut c| {
            c.inner_mut().blacklist = block_blacklist.block_render_blacklist.clone();
            c.setup = false
        });
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

pub fn chunk_update_mesh(
    query: Query<(Entity, &ChunkInstance), Changed<ChunkInstance>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, chunk_instance) in query.iter() {
        let mut mesh = Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD,
        );
        do_mesh_shit(&mut mesh, chunk_instance);
        commands.entity(entity).remove::<Handle<Mesh>>();
        commands.entity(entity).insert(meshes.add(mesh));
    }
}

pub fn lerp_rot_system(time: Res<Time>, mut query: Query<(&mut Transform, &mut LerpTransform)>) {
    for (mut transform, mut lerp_rot) in &mut query {
        lerp_rot.current_rot_time =
            (lerp_rot.current_rot_time + (time.delta_seconds() / lerp_rot.rot_time)).clamp(0., 1.);
        transform.rotation = lerp_rot
            .start_rot
            .lerp(lerp_rot.end_rot, lerp_rot.current_rot_time);
    }
}

pub fn lerp_pos_system(time: Res<Time>, mut query: Query<(&mut Transform, &mut LerpTransform)>) {
    for (mut transform, mut lerp_pos) in &mut query {
        lerp_pos.current_pos_time =
            (lerp_pos.current_pos_time + (time.delta_seconds() / lerp_pos.pos_time)).clamp(0., 1.);
        transform.translation = lerp_pos
            .start_pos
            .lerp(lerp_pos.end_pos, lerp_pos.current_pos_time);
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
    mut event: EventReader<S2CPackets>,
) {
    for msg in event.read() {
        if let S2CPackets::MovedTurtle(e) = msg {
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
