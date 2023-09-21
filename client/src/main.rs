#[allow(unused_imports)]
use bevy::log::prelude::*;
use bevy::{pbr::DirectionalLightShadowMap, prelude::*, render::texture::ImageSampler};
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiSettings};
use common::{
    client_packets::{C2SPackets, S2CPackets},
    turtle::MoveDirection,
    world_data::{get_chunk_containing_block, Chunk},
};
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

#[derive(Resource)]
pub struct WorldState {
    curr_world: Option<String>,
    worlds: Vec<String>,
}

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(WorldState {
            curr_world: None,
            worlds: Vec::new(),
        })
        .insert_resource(DirectionalLightShadowMap::default())
        .add_plugins(DefaultPlugins)
        .add_plugins(LookTransformPlugin)
        .add_plugins(OrbitCameraPlugin::new(true))
        .add_plugins(Systems)
        .add_plugins(WS)
        .add_plugins(EventsPlugin)
        .add_plugins(EguiPlugin)
        .add_event::<SpawnTurtle>()
        .add_event::<SpawnChunk>()
        .add_systems(Startup, setup)
        .add_systems(Startup, ui_setup)
        .add_systems(Update, setup_turtles)
        .add_systems(Update, turtle_spawner)
        .add_systems(Update, handle_chunk_spawning)
        .add_systems(
            Update,
            hanlde_world_updates.run_if(should_handle_world_updates),
        )
        .add_systems(Update, set_world_on_event)
        .add_systems(Update, animate_light_direction)
        .add_systems(Update, test)
        .add_systems(Update, input::orbit_input_map)
        .add_systems(Update, ui)
        .add_systems(Update, update_worlds)
        .add_systems(PreUpdate, handle_world_selection_updates)
        .run();
}

fn handle_world_selection_updates(
    worlds: Res<WorldState>,
    mut old: Local<Option<String>>,
    mut ws_writer: EventWriter<C2SPackets>,
) {
    if worlds.curr_world != *old {
        if let Some(curr) = &worlds.curr_world {
            ws_writer.send(C2SPackets::RequestWorld(curr.to_owned()));
            ws_writer.send(C2SPackets::RequestTurtles(curr.to_owned()));
        }
    }
    *old = worlds.curr_world.clone();
}

fn update_worlds(mut worlds: ResMut<WorldState>, mut ws: EventReader<S2CPackets>) {
    for p in ws.iter() {
        match p {
            S2CPackets::Worlds(w) => {
                worlds.curr_world = w.first().cloned();
                worlds.worlds = w.to_owned();
            }
            _ => (),
        }
    }
}
fn ui_setup(mut egui_settings: ResMut<EguiSettings>) {
    egui_settings.scale_factor = 1.5;
    egui_settings.sampler_descriptor = ImageSampler::default();
}

fn ui(mut worlds: ResMut<WorldState>, mut contexts: EguiContexts) {
    egui::TopBottomPanel::top("TRC").show(contexts.ctx_mut(), move |ui| {
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                let mut c = egui::ComboBox::from_label("World");
                if let Some(w) = &worlds.curr_world {
                    c = c.selected_text(w);
                }
            
                c.show_ui(ui, |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.set_min_width(60.0);
                    // Hateble (the clone here)
                    for w in worlds.worlds.clone().iter() {
                        ui.selectable_value(&mut worlds.curr_world, Some(w.to_owned()), w);
                    }
                });
            });
            ui.add(
                custom_egui_widgets::CircleDisplay::new()
                    .size(2.0)
                    .stroke_width(4.0)
                    .font_size(20.0)
                    .build(&3, &30),
            );
        });
    });
}

fn test(
    input: Res<Input<KeyCode>>,
    mut active_turtle_changed: EventWriter<ActiveTurtleChanged>,
    mut ws_writer: EventWriter<C2SPackets>,
    mut active_turtle_res: ResMut<ActiveTurtleRes>,
    turtles: Query<&TurtleInstance>,
) {
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
    ws_writer.send(C2SPackets::RequestWorlds);
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

fn should_handle_world_updates(event: EventReader<S2CPackets>) -> bool {
    let o = event.len() > 0;
    o
}

fn hanlde_world_updates(
    mut event: EventReader<S2CPackets>,
    mut query: Query<&mut ChunkInstance>,
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
                    chunk.inner_mut().set_block(block.clone());
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
