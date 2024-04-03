#![allow(clippy::too_many_arguments)]
use actually_usable_voxel_mesh_gen::util::string_to_color;
use bevy::log::{prelude::*, LogPlugin};
use bevy::{pbr::DirectionalLightShadowMap, prelude::*};
use bevy_egui::{
    egui::{self, Color32, Grid},
    EguiContexts, EguiPlugin, EguiSettings,
};
use bevy_mod_raycast::DefaultRaycastingPlugin;
use common::{
    client_packets::{C2SPackets, S2CPackets, SetTurtlesData},
    turtle::{Item, Maybe, MoveDirection},
    turtle_packets::TurtleUpDown,
    world_data::{get_chunk_containing_block, Chunk},
};
use custom_egui_widgets::item_box::{item_box, ItemSlotActions, TX};
use egui_code_editor::{CodeEditor, Syntax};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};
use trc_client::external_inv_support::ExternalInvSupportPlugin;
use std::{
    f32::consts::FRAC_PI_4,
    fs,
    ops::Deref,
    path::PathBuf,
    sync::{mpsc, Arc},
};
use trc_client::{
    bundels::ChunkBundle,
    components::ChunkInstance,
    events::{ActiveTurtleChanged, ActiveTurtleRes, EventsPlugin},
    idk::ClientChunk,
    raycast::RaycastPlugin,
    systems::Systems,
    turtle_stuff::{turtle_spawner, SpawnTurtle, TurtleInstance, TurtleModels},
    ws::WS,
    BlockBlacklist, DoBlockRaymarch, MiscState, ShowFileDialog,
};
use trc_client::{input, InputState, WorldState};


fn main() {
    pretty_env_logger::init();
    color_eyre::install().unwrap();
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
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
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "TRC".into(),
                        fit_canvas_to_parent: true,
                        // prevent_default_event_handling: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .disable::<LogPlugin>(),
        )
        .add_plugins(LookTransformPlugin)
        .add_plugins(OrbitCameraPlugin::new(true))
        .add_plugins(DefaultRaycastingPlugin)
        .add_plugins(Systems)
        .add_plugins(WS)
        .add_plugins(EventsPlugin)
        .add_plugins(EguiPlugin)
        .add_plugins(RaycastPlugin)
        .add_plugins(ExternalInvSupportPlugin)
        .add_event::<SpawnTurtle>()
        .add_event::<SpawnChunk>()
        .add_systems(Startup, setup)
        .add_systems(Startup, ui_setup)
        .add_systems(Update, setup_turtles)
        .add_systems(Update, turtle_spawner)
        .add_systems(Update, handle_chunk_spawning)
        .add_systems(
            Update,
            hanlde_world_updates.run_if(on_event::<S2CPackets>()),
        )
        .add_systems(Update, set_world_on_event)
        // .add_systems(Update, animate_light_direction)
        .add_systems(Update, test)
        .add_systems(Update, input::orbit_input_map)
        .add_systems(Update, ui)
        .add_systems(Update, update_worlds)
        .add_systems(Update, turtle_stuff_update)
        .add_systems(Update, handle_world_selection_updates)
        .add_systems(Update, file_drop)
        .add_systems(Update, fix_curr_turtle_updates)
        .run();
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
                    dialog.file = path_buf.clone();
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
    for p in ws.read() {
        if let S2CPackets::Worlds(w) = p {
            worlds.curr_world = w.first().cloned();
            worlds.worlds = w.to_owned();
        }
    }
}
fn ui_setup(mut egui_settings: ResMut<EguiSettings>) {
    egui_settings.scale_factor = 1.5;
}

fn turtle_stuff_update(
    world_state: Res<WorldState>,
    mut turtles: Query<&mut TurtleInstance>,
    mut ws_reader: EventReader<S2CPackets>,
) {
    for p in ws_reader.read() {
        match p {
            S2CPackets::TurtleInventoryUpdate(data) => {
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
                            t.inventory = data.data.clone();
                        });
                }
            }
            S2CPackets::TurtleFuelUpdate(data) => {
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
        ats.send(ActiveTurtleChanged(active_turtle_res.0))
    }
    *i = active_turtle_res.0
}

fn ui(
    mut worlds: ResMut<WorldState>,
    mut contexts: EguiContexts,
    turtles: Query<&TurtleInstance>,
    mut active_turtle_res: ResMut<ActiveTurtleRes>,
    mut input_state: ResMut<InputState>,
    mut ws_writer: EventWriter<C2SPackets>,
    egui_input: Res<bevy_egui::EguiMousePosition>,
    misc_state: Res<MiscState>,
    mut do_block_march: ResMut<DoBlockRaymarch>,
    mut item_amount_modifier: Local<u8>,
    mut file_dialog: ResMut<ShowFileDialog>,
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
    file_dialog.show &= curr_turtle.is_some();
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
                                ws.send(C2SPackets::SendLuaToTurtle {
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

    if let Some(t) = curr_turtle.as_ref() {
        let (tx, rx) = mpsc::channel();
        let window = egui::Window::new("Inventory")
            .resizable(false)
            .enabled(!file_dialog.show)
            .show(contexts.ctx_mut(), |ui| {
                let inv = t.inventory.iter().map(|i| i.to_owned()).zip(0u8..);
                let slot = t.inventory.selected_slot;
                ui.spacing_mut().interact_size.x = 0.0;
                Grid::new("inv_grid")
                    .spacing(egui::Vec2::splat(2.0))
                    .show(ui, |ui| {
                        for (item, i) in inv {
                            ui.add(ib(item, i as u32 + 1, tx.clone(), slot as u32, &mut item_amount_modifier));
                            if i % 4 == 3 {
                                ui.end_row();
                            }
                        }
                    });
            });

        while let Ok((slot, action)) = rx.try_recv() {
            match action {
                ItemSlotActions::SelectSlot => ws_writer.send(C2SPackets::TurtleSelectSlot {
                    index: t.index,
                    world: t.world.clone(),
                    slot,
                }),
                ItemSlotActions::Transfer(amount) => ws_writer.send(C2SPackets::SendLuaToTurtle {
                    index: t.index,
                    world: t.world.clone(),
                    code: format!("turtle.transferTo({slot}, {amount})"),
                }),
                ItemSlotActions::Refuel => ws_writer.send(C2SPackets::SendLuaToTurtle {
                    index: t.index,
                    world: t.world.clone(),
                    code: "turtle.refuel()".to_string(),
                }),
            }
        }
        input_state.block_camera_updates |= window
            .zip(egui_input.0)
            .is_some_and(|(w, (_, i))| w.response.rect.contains(i.to_pos2()));
    }
    if file_dialog.show {
        egui::Window::new("File Dialog")
            .collapsible(false)
            .resizable(false)
            .show(contexts.ctx_mut(), |ui| {
                ui.label(format!(
                    "Run File: \"{}\" On Current Turtle?",
                    file_dialog
                        .file
                        .file_name()
                        .map(|n| n.to_string_lossy())
                        .unwrap()
                ));
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        file_dialog.show = false;
                    }

                    if ui.button("Send").clicked() {
                        file_dialog.show = false;
                        // send shit to turtle
                        if let Some(t) = curr_turtle {
                            ws_writer.send(C2SPackets::SendLuaToTurtle {
                                index: t.index,
                                world: t.world.clone(),
                                code: file_dialog.conntents.clone(),
                            });
                            info!("Ok sending code to turtle: {}", &file_dialog.conntents);
                        }
                    }
                });
            });
    }
}

fn ib(
    item: Maybe<Item>,
    slot_id: u32,
    tx: TX,
    selected: u32,
    amount_modifier: &mut u8,
) -> impl egui::Widget + '_ {
    let item: Option<Item> = item.into();
    let color = match item.clone() {
        None => Color32::DARK_GRAY,
        Some(it) => {
            let color: [u8; 3] = string_to_color(&it.name)[0..3].try_into().unwrap();
            Color32::from_rgb(
                (color[0] >> 1) | 128u8,
                (color[1] >> 1) | 128u8,
                (color[2] >> 1) | 128u8,
            )
        }
    };
    item_box(
        item.clone().map_or(0, |i| i.count),
        item.clone().map_or("".into(), |i| i.name).into(),
        color,
        0.9,
        slot_id,
        tx.clone(),
        slot_id == selected,
        amount_modifier,
    )
}

fn test(
    input: Res<Input<KeyCode>>,
    mut ws_writer: EventWriter<C2SPackets>,
    mut contexts: EguiContexts,
    active_turtle_res: Res<ActiveTurtleRes>,
    turtles: Query<&TurtleInstance>,
    world_state: Res<WorldState>,
) {
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }
    let up_down_modifier = if input.pressed(KeyCode::ControlLeft) {
        TurtleUpDown::Down
    } else if input.pressed(KeyCode::ShiftLeft) {
        TurtleUpDown::Up
    } else {
        TurtleUpDown::Forward
    };
    let lua_func_suffix = match up_down_modifier.clone() {
        TurtleUpDown::Up => "Up",
        TurtleUpDown::Forward => "",
        TurtleUpDown::Down => "Down",
    };
    let valid_active_turtle = turtles.iter().any(|t| t.index == active_turtle_res.0);
    if input.just_pressed(KeyCode::V) && valid_active_turtle {
        ws_writer.send(C2SPackets::SendLuaToTurtle {
            world: world_state.curr_world.clone().unwrap_or_default(),
            index: active_turtle_res.0,
            code: format!("turtle.drop{lua_func_suffix}()"),
        })
    };
    if input.just_pressed(KeyCode::C) && valid_active_turtle {
        ws_writer.send(C2SPackets::SendLuaToTurtle {
            world: world_state.curr_world.clone().unwrap_or_default(),
            index: active_turtle_res.0,
            code: format!("turtle.suck{lua_func_suffix}()"),
        })
    };
    if input.just_pressed(KeyCode::F) && valid_active_turtle {
        ws_writer.send(C2SPackets::BreakBlock {
            world: world_state.curr_world.clone().unwrap_or_default(),
            index: active_turtle_res.0,
            dir: up_down_modifier.clone(),
        })
    };
    if input.just_pressed(KeyCode::R) && valid_active_turtle {
        ws_writer.send(C2SPackets::PlaceBlock {
            world: world_state.curr_world.clone().unwrap_or_default(),
            index: active_turtle_res.0,
            dir: up_down_modifier.clone(),
            text: None,
        })
    };
    if input.just_pressed(KeyCode::W) && valid_active_turtle {
        ws_writer.send(C2SPackets::MoveTurtle {
            world: world_state.curr_world.clone().unwrap_or_default(),
            index: active_turtle_res.0,
            direction: MoveDirection::Forward,
        })
    };
    if input.just_pressed(KeyCode::S) && valid_active_turtle {
        ws_writer.send(C2SPackets::MoveTurtle {
            world: world_state.curr_world.clone().unwrap_or_default(),
            index: active_turtle_res.0,
            direction: MoveDirection::Back,
        })
    };
    if input.just_pressed(KeyCode::A) && valid_active_turtle {
        ws_writer.send(C2SPackets::MoveTurtle {
            world: world_state.curr_world.clone().unwrap_or_default(),
            index: active_turtle_res.0,
            direction: MoveDirection::Left,
        })
    };
    if input.just_pressed(KeyCode::D) && valid_active_turtle {
        ws_writer.send(C2SPackets::MoveTurtle {
            world: world_state.curr_world.clone().unwrap_or_default(),
            index: active_turtle_res.0,
            direction: MoveDirection::Right,
        })
    };
    if input.just_pressed(KeyCode::E) && valid_active_turtle {
        ws_writer.send(C2SPackets::MoveTurtle {
            world: world_state.curr_world.clone().unwrap_or_default(),
            index: active_turtle_res.0,
            direction: MoveDirection::Up,
        })
    };
    if input.just_pressed(KeyCode::Q) && valid_active_turtle {
        ws_writer.send(C2SPackets::MoveTurtle {
            world: world_state.curr_world.clone().unwrap_or_default(),
            index: active_turtle_res.0,
            direction: MoveDirection::Down,
        })
    };
}

fn setup_turtles(
    mut spwan_turtle: EventWriter<SpawnTurtle>,
    mut ws_reader: EventReader<S2CPackets>,
    world_state: Res<WorldState>,
    active_turtle_res: Res<ActiveTurtleRes>,
    query: Query<Entity, With<TurtleInstance>>,
    mut commands: Commands,
) {
    for p in ws_reader.read() {
        if let S2CPackets::SetTurtles(SetTurtlesData { turtles, world }) = p.to_owned() {
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
    mut ws_writer: EventWriter<C2SPackets>,
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
    commands.insert_resource(ChunkMat(materials.add(Color::rgb(1., 1., 1.).into())));
    ws_writer.send(C2SPackets::RequestWorlds);
}

fn set_world_on_event(
    query: Query<Entity, With<ChunkInstance>>,
    mut commands: Commands,
    mut event: EventReader<S2CPackets>,
    mut chunk_spawn: EventWriter<SpawnChunk>,
) {
    for e in event.read() {
        if let S2CPackets::SetWorld(world) = e {
            query.for_each(|entity| {
                commands.entity(entity).despawn_recursive();
            });
            world
                .get_chunks()
                .iter()
                .for_each(|(_, chunk)| chunk_spawn.send(SpawnChunk(chunk.clone())))
        }
    }
}

fn hanlde_world_updates(
    mut event: EventReader<S2CPackets>,
    mut query: Query<&mut ChunkInstance>,
    mut chunk_spawn: EventWriter<SpawnChunk>,
) {
    for e in event.read() {
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
    for e in event.read() {
        commands.spawn(ChunkBundle::new(
            ClientChunk::from_chunk(e.0.clone()),
            &mut meshes,
            chunk_mat.clone(),
        ));
    }
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
