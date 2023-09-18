#[allow(unused_imports)]
use bevy::log::prelude::*;
use bevy::{pbr::DirectionalLightShadowMap, prelude::*};
use common::{
    client_packets::{C2SPackets, S2CPackets},
    turtle::MoveDirection,
    world_data::{get_chunk_containing_block, Chunk, World},
};
use futures::executor::block_on;
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};
use std::f32::consts::FRAC_PI_4;
use trc_client::input;
use trc_client::{
    bundels::ChunkBundle,
    components::ChunkInstance,
    events::{ActiveTurtleChanged, ActiveTurtleRes, EventsPlugin},
    idk::ClientChunk,
    systems::Systems,
    turtle_stuff::{turtle_spawner, SpawnTurtle, TurtleInstance, TurtleModels},
    ws::WS,
};

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
        .add_event::<SpawnChunk>()
        .add_systems(Startup, setup)
        .add_systems(Update, setup_turtles)
        .add_systems(Update, turtle_spawner)
        .add_systems(Update, handle_chunk_spawning)
        .add_systems(Update, hanlde_world_updates)
        .add_systems(Update, set_world_on_event)
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
            S2CPackets::SetTurtles(ts) => {
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
#[derive(Component)]
pub struct MainCamera;
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ws_writer: EventWriter<C2SPackets>,
) {
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
        ))
        .insert(MainCamera);

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
    commands.insert_resource(ChunkMat(materials.add(Color::rgb(1., 1., 1.).into())));
    ws_writer.send(C2SPackets::RequestTurtles);
    ws_writer.send(C2SPackets::RequestWorld("test_world_01".into()));
}

fn set_world_on_event(
    query: Query<Entity, With<ChunkInstance>>,
    mut commands: Commands,
    mut event: EventReader<S2CPackets>,
    mut chunk_spawn: EventWriter<SpawnChunk>,
) {
    for e in event.iter() {
        if let S2CPackets::SetWorld(world) = e {
            query.for_each(|entity| {
                commands.entity(entity).despawn();
            });
            world
                .get_chunks()
                .iter()
                .for_each(|(_, chunk)| chunk_spawn.send(SpawnChunk(chunk.clone())))
        }
    }
}

fn hanlde_world_updates(
    mut query: Query<&mut ChunkInstance>,
    mut event: EventReader<S2CPackets>,
    mut chunk_spawn: EventWriter<SpawnChunk>,
) {
    for e in event.iter() {
        if let S2CPackets::WorldUpdate(block) = e {
            let chunk_pos = get_chunk_containing_block(block.get_pos());
            match query
                .iter_mut()
                .find(|chunk| chunk.get_chunk_pos() == &chunk_pos)
            {
                Some(mut chunk) => {
                    info!("pos: {:#?}", block.get_name());
                    chunk.set_block(block.clone());
                }
                None => {
                    let mut chunk = Chunk::new(chunk_pos);
                    chunk.set_block(block.clone());
                    chunk_spawn.send(SpawnChunk(chunk));
                }
            }
        }
    }
}

#[derive(Resource, Debug, DerefMut, Deref)]
struct ChunkMat(Handle<StandardMaterial>);

#[derive(Event, Debug, Deref, DerefMut)]
struct SpawnChunk(Chunk);

fn handle_chunk_spawning(
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut event: EventReader<SpawnChunk>,
    chunk_mat: Res<ChunkMat>,
) {
    for e in event.iter() {
        commands.spawn(ChunkBundle::new(
            ClientChunk::from_chunk(e.0.clone()),
            &mut meshes,
            chunk_mat.clone(),
        ));
    }
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
