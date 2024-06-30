#![allow(clippy::too_many_arguments)]
use bevy::{pbr::DirectionalLightShadowMap, prelude::*};
use bevy_egui::{
    egui::{self, Grid},
    EguiContexts, EguiPlugin, EguiSettings,
};
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_mod_raycast::DefaultRaycastingPlugin;
use bevy_schminput::DefaultSchmugins;
use common::{
    client_packets::{C2SPacket, S2CPacket, SetTurtlesData},
    turtle::{Maybe, Orientation},
    world_data::{get_chunk_containing_block, Block, Chunk},
    Pos3,
};
use custom_egui_widgets::item_box::SlotInteraction;
use egui_code_editor::{CodeEditor, Syntax};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};
use std::{
    f32::consts::FRAC_PI_4,
    fs,
    ops::Deref,
    path::PathBuf,
    sync::{mpsc, Arc},
};
use trc_client::{
    events::{ActiveTurtleChanged, ActiveTurtleRes, EventsPlugin},
    primary_ui::PrimaryUiPlugin,
    raycast::RaycastPlugin,
    systems::Systems,
    turtle::LocallyControlledTurtle,
    turtle_input::TurtleInputPlugin,
    turtle_stuff::{turtle_spawner, SpawnTurtle, TurtleInstance, TurtleModels},
    ws::WS,
    BlockBlacklist, DoBlockRaymarch, MiscState, ShowFileDialog,
};
use trc_client::{
    executable_files::ExecutableFilesPlugin,
    lerp_transform::LerpTransformPlugin,
    turtle::{TurtleBundle, TurtlePlugin},
    turtle_inventory::TurtleInventoryPlugin,
    turtle_movement::TurtleMovementPlugin,
};
use trc_client::{external_inv_support::ExternalInvSupportPlugin, turtle};
use trc_client::{input, InputState, WorldState};

fn main() {
    // pretty_env_logger::init();
    color_eyre::install().unwrap();
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "TRC".into(),

                    // fit_canvas_to_parent: true,
                    // prevent_default_event_handling: true,
                    ..Default::default()
                }),
                ..Default::default()
            }), // .disable::<LogPlugin>(),
        )
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(DefaultSchmugins)
        .add_plugins(LerpTransformPlugin)
        .add_plugins(ExecutableFilesPlugin)
        .add_plugins(LookTransformPlugin)
        .add_plugins(OrbitCameraPlugin::new(true))
        .add_plugins(DefaultRaycastingPlugin)
        .add_plugins(Systems)
        .add_plugins(WS)
        .add_plugins(EventsPlugin)
        .add_plugins(EguiPlugin)
        .add_plugins(RaycastPlugin)
        .add_plugins(ExternalInvSupportPlugin)
        .add_plugins(TurtlePlugin)
        .add_plugins(TurtleMovementPlugin)
        .add_plugins(TurtleInputPlugin)
        .add_plugins(TurtleInventoryPlugin)
        .add_plugins(PrimaryUiPlugin)
        .add_event::<SpawnTurtle>()
        .add_event::<SpawnChunk>()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            ..Default::default()
        })
        .insert_resource(WorldState {
            curr_world: None,
            worlds: Vec::new(),
        })
        .insert_resource(InputState {
            block_camera_updates: false,
        })
        .insert_resource(BlockBlacklist {
            block_render_blacklist: Arc::new([
                "minecraft:water".into(),
                "computercraft:turtle_normal".into(),
                "computercraft:turtle_advanced".into(),
            ]),
        })
        .insert_resource(MiscState {
            hovered_block: None,
        })
        .insert_resource(ShowFileDialog {
            show: false,
            conntents: "".into(),
            file: PathBuf::default(),
        })
        .insert_resource(DoBlockRaymarch(true))
        .insert_resource(DirectionalLightShadowMap { size: 1024 * 4 })
        .add_systems(Startup, setup)
        .add_systems(Startup, ui_setup)
        .add_systems(Update, setup_turtles)
        .add_systems(Update, turtle_spawner)
        .add_systems(Update, handle_chunk_spawning)
        // .add_systems(Update, hanlde_world_updates.run_if(on_event::<S2CPacket>()))
        // .add_systems(Update, set_world_on_event)
        // .add_systems(Update, animate_light_direction)
        .add_systems(Update, test)
        .add_systems(Update, input::orbit_input_map)
        // .add_systems(Update, ui)
        .add_systems(Update, update_worlds)
        .add_systems(Update, turtle_stuff_update)
        .add_systems(Update, handle_world_selection_updates)
        .add_systems(Update, file_drop)
        .add_systems(Update, fix_curr_turtle_updates)
        .add_systems(PostStartup, spawn_dummy_turtle)
        // .add_systems(PostStartup, set_fake_block)
        .run();
}
// fn set_fake_block(mut query: Query<&mut ChunkInstance>, mut chunk_spawn: EventWriter<SpawnChunk>) {
//     let block = Block::new(Some("minecraft:dirt".into()), &Pos3::NEG_Y, "test_world");
//     let chunk_pos = get_chunk_containing_block(block.get_pos());
//     match query
//         .iter_mut()
//         .find(|chunk| chunk.get_chunk_pos() == &chunk_pos)
//     {
//         Some(mut chunk) => {
//             chunk.inner_mut().set_block(block.clone());
//         }
//         None => {
//             let mut chunk = Chunk::new(chunk_pos);
//             chunk.set_block(block.clone());
//             chunk_spawn.send(SpawnChunk(chunk));
//         }
//     }
// }

fn spawn_dummy_turtle(mut commands: Commands, models: Res<turtle::TurtleModels>) {
    commands
        .spawn(TurtleBundle::from_common_turtle(
            common::turtle::Turtle {
                index: 1,
                name: "Test".into(),
                inventory: Maybe::None,
                position: Pos3::ZERO,
                orientation: Orientation::North,
                fuel: 20,
                max_fuel: 360,
                is_online: true,
                world: "test_world".into(),
            },
            &models,
        ))
        .insert(LocallyControlledTurtle);
}

fn file_drop(mut dnd_evr: EventReader<FileDragAndDrop>, mut dialog: ResMut<ShowFileDialog>) {
    for ev in dnd_evr.read() {
        if let FileDragAndDrop::DroppedFile {
            window: _,
            path_buf,
        } = ev
        {
            match fs::read_to_string(path_buf) {
                Ok(text) => {
                    dialog.file.clone_from(path_buf);
                    dialog.show = true;
                    dialog.conntents = text;
                }
                Err(_err) => (),
            }
        }
    }
}

fn handle_world_selection_updates(
    worlds: Res<WorldState>,
    mut old: Local<Option<String>>,
    mut ws_writer: EventWriter<C2SPacket>,
) {
    if worlds.curr_world != *old {
        if let Some(curr) = &worlds.curr_world {
            ws_writer.send(C2SPacket::SwitchWorld(Maybe::Some(curr.clone())));
            ws_writer.send(C2SPacket::RequestWorld);
            ws_writer.send(C2SPacket::RequestTurtles);
        }
    }
    old.clone_from(&worlds.curr_world);
}

fn update_worlds(mut worlds: ResMut<WorldState>, mut ws: EventReader<S2CPacket>) {
    for p in ws.read() {
        if let S2CPacket::Worlds(w) = p {
            worlds.curr_world = w.first().cloned();
            w.clone_into(&mut worlds.worlds);
        }
    }
}

fn ui_setup(mut egui_settings: ResMut<EguiSettings>) {
    egui_settings.scale_factor = 1.5;
}

fn turtle_stuff_update(
    world_state: Res<WorldState>,
    mut turtles: Query<&mut TurtleInstance>,
    mut ws_reader: EventReader<S2CPacket>,
) {
    for p in ws_reader.read() {
        match p {
            S2CPacket::TurtleInventoryUpdate(data) => {
                if world_state
                    .curr_world
                    .as_ref()
                    .is_some_and(|w| w == &data.world)
                {
                    turtles
                        .iter_mut()
                        .find(|t| t.index == data.index)
                        .iter_mut()
                        .for_each(|t| {
                            t.inventory = Some(data.data.clone()).into();
                        });
                }
            }
            S2CPacket::TurtleFuelUpdate(data) => {
                if world_state
                    .curr_world
                    .as_ref()
                    .is_some_and(|w| w == &data.world)
                {
                    turtles
                        .iter_mut()
                        .find(|t| t.index == data.index)
                        .iter_mut()
                        .for_each(|t| {
                            t.fuel = data.data;
                        });
                }
            }
            _ => (),
        }
    }
}

fn fix_curr_turtle_updates(
    active_turtle_res: Res<ActiveTurtleRes>,
    mut i: Local<i32>,
    mut ats: EventWriter<ActiveTurtleChanged>,
) {
    if *i != active_turtle_res.0 {
        ats.send(ActiveTurtleChanged(active_turtle_res.0));
    }
    *i = active_turtle_res.0
}

fn ui(
    mut worlds: ResMut<WorldState>,
    mut contexts: EguiContexts,
    turtles: Query<&TurtleInstance>,
    mut active_turtle_res: ResMut<ActiveTurtleRes>,
    mut input_state: ResMut<InputState>,
    mut ws_writer: EventWriter<C2SPacket>,
    egui_input: Res<bevy_egui::EguiMousePosition>,
    misc_state: Res<MiscState>,
    mut do_block_march: ResMut<DoBlockRaymarch>,
    mut item_amount_modifier: Local<u8>,
    mut lua_code_str: Local<String>,
) {
    if **do_block_march && !input_state.block_camera_updates {
        if let Some(b) = misc_state.hovered_block.as_ref() {
            egui::show_tooltip_text(
                contexts.ctx_mut(),
                egui::Id::new("block_raycast_tooltip"),
                b,
            );
        }
    }
    input_state.block_camera_updates = false;
    let online_turtles = turtles
        .iter()
        .filter(|t| t.turtle.is_online)
        .map(|t| t.deref().clone())
        .collect::<Vec<_>>();
    let why = online_turtles.clone();
    let curr_turtle = why.iter().find(|t| t.index == active_turtle_res.0);
    let ws = &mut ws_writer;
    let main_panel = egui::TopBottomPanel::top("TRC").show(contexts.ctx_mut(), move |ui| {
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                // ui.spacing_mut().interact_size.y *= 1.5;
                ui.checkbox(&mut do_block_march, "Block Raycast");
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
                // Turtles
                let mut c = egui::ComboBox::from_label("Turtle");
                if let Some(t) = &curr_turtle {
                    c = c.selected_text(format!("{}: {}", &t.index, &t.name));
                }

                c.show_ui(ui, |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.set_min_width(60.0);
                    for t in online_turtles.iter() {
                        ui.selectable_value(
                            &mut active_turtle_res.0,
                            t.index,
                            &format!("{}: {}", &t.index, &t.name),
                        );
                    }
                });
            });
            if let Some(t) = curr_turtle {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.add_space(1.0);
                    ui.add(
                        custom_egui_widgets::CircleDisplay::new()
                            .size(3.0)
                            .stroke_width(6.0)
                            .font_size(12.0)
                            .build(&t.fuel, &t.max_fuel),
                    );
                    ui.add_space(1.0);
                    ui.label(format!(
                        "Turtle Pos: x {}, y {}, z {}",
                        t.position.x, t.position.y, t.position.z
                    ));
                    ui.menu_button("Lua Code Console", |ui| {
                        CodeEditor::default()
                            .id_source("Lua Console Editor")
                            .with_syntax(Syntax::lua())
                            .show(ui, &mut lua_code_str);
                        ui.horizontal(|ui| {
                            if ui.button("Submit").clicked() {
                                ws.send(C2SPacket::SendLuaToTurtle {
                                    index: t.index,
                                    world: t.world.clone(),
                                    code: (*lua_code_str).clone(),
                                });
                                ui.close_menu();
                            }
                            if ui.button("Close").clicked() {
                                ui.close_menu();
                            }
                        });
                    });
                });
            }
        });
    });

    input_state.block_camera_updates |= main_panel.response.hovered();

    // if let Some(t) = curr_turtle.as_ref() {
    //     let (tx, rx) = mpsc::channel();
    //     if let Maybe::Some(inv) = t.inventory.clone() {
    //         let window = egui::Window::new("Inventory")
    //             .resizable(false)
    //             // .enabled(!file_dialog.show)
    //             .show(contexts.ctx_mut(), |ui| {
    //                 let inv = inv.iter().map(|i| i.to_owned()).zip(0u8..);
    //                 let slot = t.inventory.clone().unwrap().selected_slot;
    //                 ui.spacing_mut().interact_size.x = 0.0;
    //                 Grid::new("inv_grid")
    //                     .spacing(egui::Vec2::splat(2.0))
    //                     .show(ui, |ui| {
    //                         for (item, i) in inv {
    //                             ui.add(ib(
    //                                 item,
    //                                 i as u32 + 1,
    //                                 tx.clone(),
    //                                 slot as u32,
    //                                 &mut item_amount_modifier,
    //                             ));
    //                             if i % 4 == 3 {
    //                                 ui.end_row();
    //                             }
    //                         }
    //                     });
    //             });
    //
    //         while let Ok((slot, action)) = rx.try_recv() {
    //             match action {
    //                 // ItemSlotActions::SelectSlot => ws_writer.send(C2SPackets::TurtleSelectSlot {
    //                 //     index: t.index,
    //                 //     world: t.world.clone(),
    //                 //     slot,
    //                 // }),
    //                 SlotInteraction::Transfer(amount) => {
    //                     ws_writer.send(C2SPacket::SendLuaToTurtle {
    //                         index: t.index,
    //                         world: t.world.clone(),
    //                         code: format!("turtle.transferTo({slot}, {amount})"),
    //                     });
    //                 }
    //                 SlotInteraction::Refuel => {
    //                     ws_writer.send(C2SPacket::SendLuaToTurtle {
    //                         index: t.index,
    //                         world: t.world.clone(),
    //                         code: "turtle.refuel()".to_string(),
    //                     });
    //                 }
    //                 _ => (),
    //             };
    //         }
    //         input_state.block_camera_updates |= window
    //             .zip(egui_input.0)
    //             .is_some_and(|(w, (_, i))| w.response.rect.contains(i.to_pos2()));
    //     }
    // }
}

fn test(
    input: Res<ButtonInput<KeyCode>>,
    mut ws_writer: EventWriter<C2SPacket>,
    mut contexts: EguiContexts,
    active_turtle_res: Res<ActiveTurtleRes>,
    turtles: Query<&TurtleInstance>,
    world_state: Res<WorldState>,
) {
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }
    // let up_down_modifier = if input.pressed(KeyCode::ControlLeft) {
    //     TurtleUpDown::Down
    // } else if input.pressed(KeyCode::ShiftLeft) {
    //     TurtleUpDown::Up
    // } else {
    //     TurtleUpDown::Forward
    // };
    // let lua_func_suffix = match up_down_modifier.clone() {
    //     TurtleUpDown::Up => "Up",
    //     TurtleUpDown::Forward => "",
    //     TurtleUpDown::Down => "Down",
    // };
    let valid_active_turtle = turtles.iter().any(|t| t.index == active_turtle_res.0);
    // if input.just_pressed(KeyCode::KeyV) && valid_active_turtle {
    //     ws_writer.send(C2SPackets::SendLuaToTurtle {
    //         world: world_state.curr_world.clone().unwrap_or_default(),
    //         index: active_turtle_res.0,
    //         code: format!("turtle.drop{lua_func_suffix}()"),
    //     });
    // };
    // if input.just_pressed(KeyCode::KeyC) && valid_active_turtle {
    //     ws_writer.send(C2SPackets::SendLuaToTurtle {
    //         world: world_state.curr_world.clone().unwrap_or_default(),
    //         index: active_turtle_res.0,
    //         code: format!("turtle.suck{lua_func_suffix}()"),
    //     });
    // };
    // if input.just_pressed(KeyCode::KeyF) && valid_active_turtle {
    //     ws_writer.send(C2SPackets::BreakBlock {
    //         world: world_state.curr_world.clone().unwrap_or_default(),
    //         index: active_turtle_res.0,
    //         dir: up_down_modifier.clone(),
    //     });
    // };
    // if input.just_pressed(KeyCode::KeyR) && valid_active_turtle {
    //     ws_writer.send(C2SPackets::PlaceBlock {
    //         world: world_state.curr_world.clone().unwrap_or_default(),
    //         index: active_turtle_res.0,
    //         dir: up_down_modifier.clone(),
    //         text: None,
    //     });
    // };
    // if input.just_pressed(KeyCode::KeyW) && valid_active_turtle {
    //     ws_writer.send(C2SPackets::MoveTurtle {
    //         world: world_state.curr_world.clone().unwrap_or_default(),
    //         index: active_turtle_res.0,
    //         direction: MoveDirection::Forward,
    //     });
    // };
    // if input.just_pressed(KeyCode::KeyS) && valid_active_turtle {
    //     ws_writer.send(C2SPackets::MoveTurtle {
    //         world: world_state.curr_world.clone().unwrap_or_default(),
    //         index: active_turtle_res.0,
    //         direction: MoveDirection::Back,
    //     });
    // };
    // if input.just_pressed(KeyCode::KeyA) && valid_active_turtle {
    //     ws_writer.send(C2SPackets::MoveTurtle {
    //         world: world_state.curr_world.clone().unwrap_or_default(),
    //         index: active_turtle_res.0,
    //         direction: MoveDirection::Left,
    //     });
    // };
    // if input.just_pressed(KeyCode::KeyD) && valid_active_turtle {
    //     ws_writer.send(C2SPackets::MoveTurtle {
    //         world: world_state.curr_world.clone().unwrap_or_default(),
    //         index: active_turtle_res.0,
    //         direction: MoveDirection::Right,
    //     });
    // };
    // if input.just_pressed(KeyCode::KeyE) && valid_active_turtle {
    //     ws_writer.send(C2SPackets::MoveTurtle {
    //         world: world_state.curr_world.clone().unwrap_or_default(),
    //         index: active_turtle_res.0,
    //         direction: MoveDirection::Up,
    //     });
    // };
    // if input.just_pressed(KeyCode::KeyQ) && valid_active_turtle {
    //     ws_writer.send(C2SPackets::MoveTurtle {
    //         world: world_state.curr_world.clone().unwrap_or_default(),
    //         index: active_turtle_res.0,
    //         direction: MoveDirection::Down,
    //     });
    // };
}

fn setup_turtles(
    mut spwan_turtle: EventWriter<SpawnTurtle>,
    mut ws_reader: EventReader<S2CPacket>,
    world_state: Res<WorldState>,
    active_turtle_res: Res<ActiveTurtleRes>,
    query: Query<Entity, With<TurtleInstance>>,
    mut commands: Commands,
) {
    for p in ws_reader.read() {
        if let S2CPacket::SetTurtles(SetTurtlesData { turtles, world }) = p.to_owned() {
            if world_state.curr_world.as_ref().is_some_and(|w| w == &world) {
                query.iter().for_each(|entity| {
                    commands.entity(entity).despawn_recursive();
                });
                turtles.into_iter().for_each(|t| {
                    spwan_turtle.send(SpawnTurtle {
                        active: active_turtle_res.0 == t.index && t.is_online,
                        turtle: t,
                    });
                });
            }
        }
    }
}
#[derive(Component)]
pub struct MainCamera;
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ws_writer: EventWriter<C2SPacket>,
) {
    // #[cfg(not(target_arch = "wasm32"))]
    // let zoomies = 1.0;
    // #[cfg(target_arch = "wasm32")]
    let zoomies = 0.1;
    commands
        .spawn(Camera3dBundle::default())
        .insert(OrbitCameraBundle::new(
            OrbitCameraController {
                mouse_rotate_sensitivity: Vec2::splat(0.25),
                mouse_wheel_zoom_sensitivity: zoomies,
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
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            FRAC_PI_4,
            -FRAC_PI_4,
        )),
        ..default()
    });
    commands.insert_resource(TurtleModels {
        active_turtle: asset_server.load("turtle.gltf#Scene0"),
        inactive_turtle: asset_server.load("turtle_inactive.gltf#Scene0"),
    });
    commands.insert_resource(ActiveTurtleRes(0));
    commands.insert_resource(ChunkMat(
        materials.add(StandardMaterial::from(Color::rgb(1., 1., 1.))),
    ));
    ws_writer.send(C2SPacket::RequestWorld);
}

// fn set_world_on_event(
//     query: Query<Entity, With<ChunkInstance>>,
//     mut commands: Commands,
//     mut event: EventReader<S2CPacket>,
//     mut chunk_spawn: EventWriter<SpawnChunk>,
// ) {
//     for e in event.read() {
//         if let S2CPacket::SetWorld(world) = e {
//             query.iter().for_each(|entity| {
//                 commands.entity(entity).despawn_recursive();
//             });
//             world.get_chunks().iter().for_each(|(_, chunk)| {
//                 chunk_spawn.send(SpawnChunk(chunk.clone()));
//             })
//         }
//     }
// }

// fn hanlde_world_updates(
//     mut event: EventReader<S2CPacket>,
//     mut query: Query<&mut ChunkInstance>,
//     mut chunk_spawn: EventWriter<SpawnChunk>,
// ) {
//     for e in event.read() {
//         if let S2CPacket::WorldUpdate(block) = e {
//             let chunk_pos = get_chunk_containing_block(block.get_pos());
//             match query
//                 .iter_mut()
//                 .find(|chunk| chunk.get_chunk_pos() == &chunk_pos)
//             {
//                 Some(mut chunk) => {
//                     chunk.inner_mut().set_block(block.clone());
//                 }
//                 None => {
//                     let mut chunk = Chunk::new(chunk_pos);
//                     chunk.set_block(block.clone());
//                     chunk_spawn.send(SpawnChunk(chunk));
//                 }
//             }
//         }
//     }
// }

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
    // for e in event.read() {
    //     commands.spawn(ChunkBundle::new(
    //         ClientChunk::from_chunk(e.0.clone()),
    //         &mut meshes,
    //         chunk_mat.clone(),
    //     ));
    // }
}

// fn animate_light_direction(
//     time: Res<Time>,
//     mut query: Query<&mut Transform, With<DirectionalLight>>,
// ) {
//     for mut transform in &mut query {
//         transform.rotation = Quat::from_euler(
//             EulerRot::ZYX,
//             0.0,
//             time.elapsed_seconds() * std::f32::consts::PI / 5.0,
//             -FRAC_PI_4,
//         );
//     }
// }
