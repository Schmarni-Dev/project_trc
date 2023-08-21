use bevy::{log::prelude::*, pbr::DirectionalLightShadowMap, prelude::*};
use trc_client::{
    bundels::ChunkBundle,
    events::{ActiveTurtleChanged, ActiveTurtleRes, EventsPlugin},
    idk::ClientChunk,
    systems::Systems,
    turtle_stuff::{turtle_spawner, SpawnTurtle, TurtleInstance, TurtleModels},
    ws::{WsCommunicator, WS},
    *,
};
use common::{
    client_packets::{C2SPackets, S2CPackets},
    turtle::MoveDirection,
    Pos3,
};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};
use trc_client::input;
use std::f32::consts::FRAC_PI_4;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(DirectionalLightShadowMap::default())
        .add_plugins(DefaultPlugins)
        .add_plugins(LookTransformPlugin)
        .add_plugins(OrbitCameraPlugin::new(true))
        .add_plugins(Systems)
        .add_plugins(WS)
        .add_plugins(EventsPlugin)
        .add_event::<SpawnTurtle>()
        .add_systems(Startup, setup)
        .add_systems(Update, setup_turtles)
        .add_systems(Update, turtle_spawner)
        .add_systems(Update, animate_light_direction)
        .add_systems(Update, test)
        .add_systems(Update, input::orbit_input_map)
        .run();
}

fn test(
    input: Res<Input<KeyCode>>,
    mut active_turtle_changed: EventWriter<ActiveTurtleChanged>,
    mut ws_writer: EventWriter<C2SPackets>,
    mut active_turtle_res: ResMut<ActiveTurtleRes>,
    turtles: Query<&TurtleInstance>,
    mut commands: Commands,
) {
    // if input.just_pressed(KeyCode::T) {
    //     info!("RESETING THE WS CONNECTION!!!");
    //     let ws_communitcator = WsCommunicator::init("ws://localhost:9001");
    //     commands.insert_resource(ws_communitcator);
    // };
    if input.just_pressed(KeyCode::Period) {
        active_turtle_res.0 += 1;
        active_turtle_changed.send(ActiveTurtleChanged(active_turtle_res.0));
    };
    if input.just_pressed(KeyCode::Comma) {
        active_turtle_res.0 -= 1;
        active_turtle_changed.send(ActiveTurtleChanged(active_turtle_res.0));
    };
    let valid_active_turtle = turtles.iter().any(|t| t.index == active_turtle_res.0);
    if input.just_pressed(KeyCode::W) && valid_active_turtle {
        ws_writer.send(C2SPackets::MoveTurtle {
            index: active_turtle_res.0,
            direction: MoveDirection::Forward,
        })
    };
    if input.just_pressed(KeyCode::S) && valid_active_turtle {
        ws_writer.send(C2SPackets::MoveTurtle {
            index: active_turtle_res.0,
            direction: MoveDirection::Back,
        })
    };
    if input.just_pressed(KeyCode::A) && valid_active_turtle {
        ws_writer.send(C2SPackets::MoveTurtle {
            index: active_turtle_res.0,
            direction: MoveDirection::Left,
        })
    };
    if input.just_pressed(KeyCode::D) && valid_active_turtle {
        ws_writer.send(C2SPackets::MoveTurtle {
            index: active_turtle_res.0,
            direction: MoveDirection::Right,
        })
    };
    if input.just_pressed(KeyCode::E) && valid_active_turtle {
        ws_writer.send(C2SPackets::MoveTurtle {
            index: active_turtle_res.0,
            direction: MoveDirection::Up,
        })
    };
    if input.just_pressed(KeyCode::Q) && valid_active_turtle {
        ws_writer.send(C2SPackets::MoveTurtle {
            index: active_turtle_res.0,
            direction: MoveDirection::Down,
        })
    };
}

fn setup_turtles(
    mut spwan_turtle: EventWriter<SpawnTurtle>,
    mut ws_reader: EventReader<S2CPackets>,
    active_turtle_res: Res<ActiveTurtleRes>,
) {
    for p in ws_reader.iter() {
        match p.to_owned() {
            S2CPackets::RequestedTurtles(ts) => {
                ts.into_iter().for_each(|t| {
                    spwan_turtle.send(SpawnTurtle {
                        active: active_turtle_res.0 == t.index,
                        turtle: t,
                    });
                });
            }
            S2CPackets::TurtleConnected(t) => {
                spwan_turtle.send(SpawnTurtle {
                    active: active_turtle_res.0 == t.index,
                    turtle: t,
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
    commands.insert_resource(ActiveTurtleRes(0));

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
