use bevy::{pbr::DirectionalLightShadowMap, prelude::*};
use client::{bundels::ChunkBundle, components::LerpPos, idk::ClientChunk, systems::Systems, *};
use common::Pos3;
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
        .insert_resource(DirectionalLightShadowMap { size: 8192 })
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::new(true))
        .add_plugin(Systems)
        .add_startup_system(setup)
        .add_system(animate_light_direction)
        .add_system(input::orbit_input_map)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("turtle.gltf#Scene0"),
            transform: Transform::from_translation(Vec3::splat(0.5)),
            ..default()
        })
        .insert(LerpPos::new(
            Vec3::splat(0.5),
            Vec3::new(0.5, 1.5, 0.5),
            2.5,
        ));
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
