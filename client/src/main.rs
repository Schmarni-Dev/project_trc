use bevy::{log::prelude::*, pbr::DirectionalLightShadowMap, prelude::*};
use client::{
    bundels::ChunkBundle,
    idk::ClientChunk,
    systems::Systems,
    turtle_stuff::{turtle_spawner, TurtleModels, TurtleSpawnData},
    ws::WS,
    *,
};
use common::{
    client_packets::{C2SPackets, S2CPackets},
    Pos3,
};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};
use std::f32::consts::FRAC_PI_4;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(DirectionalLightShadowMap::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::new(true))
        .add_plugin(Systems)
        .add_plugin(WS)
        .add_event::<TurtleSpawnData>()
        .add_startup_system(setup)
        .add_system(setup_turtles)
        .add_system(turtle_spawner)
        .add_system(animate_light_direction)
        .add_system(input::orbit_input_map)
        .run();
}

fn setup_turtles(
    mut spwan_turtle: EventWriter<TurtleSpawnData>,
    mut ws_reader: EventReader<S2CPackets>,
) {
    for p in ws_reader.iter() {
        info!("test");
        match p.to_owned() {
            S2CPackets::RequestedTurtles(ts) => {
                ts.into_iter().for_each(|t| {
                    spwan_turtle.send(TurtleSpawnData {
                        turtle: t,
                        active: false,
                    });
                });
            }
            _ => {}
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ws_writer: EventWriter<C2SPackets>,
) {
    let mut chunk_1 = ClientChunk::new(Pos3::ZERO);
    chunk_1.add_block(Pos3::new(0, 0, 0), "origin");
    chunk_1.add_block(Pos3::new(1, 0, 0), "Test");
    chunk_1.add_block(Pos3::new(15, 0, 0), "sus:among_us");
    let mut chunk_2 = ClientChunk::new(Pos3::new(1, 0, 0));
    chunk_2.add_block(Pos3::new(0, 1, 0), "minecraft:moss_block");
    chunk_2.add_block(Pos3::new(1, 1, 0), "minecraft:grass_block");
    commands
        .spawn(Camera3dBundle::default())
        .insert(OrbitCameraBundle::new(
            OrbitCameraController {
                mouse_rotate_sensitivity: Vec2::splat(0.25),
                ..default()
            },
            Vec3::splat(2.),
            Vec3::splat(0.5),
            Vec3::Y,
        ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });
    commands.insert_resource(TurtleModels {
        active_turtle: asset_server.load("turtle.gltf#Scene0"),
        inactive_turtle: asset_server.load("turtle_inactive.gltf#Scene0"),
    });

    ws_writer.send(C2SPackets::RequestTurtles);

    let chunk_mat = materials.add(Color::rgb(1., 1., 1.).into());
    commands.spawn(ChunkBundle::new(chunk_1, &mut meshes, chunk_mat.clone()));
    commands.spawn(ChunkBundle::new(chunk_2, &mut meshes, chunk_mat.clone()));
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * std::f32::consts::PI / 5.0,
            -FRAC_PI_4,
        );
    }
}
