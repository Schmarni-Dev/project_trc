use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use common::client_packets::C2SPacket;

use crate::{events::ActiveTurtleRes, turtle_stuff::TurtleInstance, ShowFileDialog};

pub struct ExecutableFilesPlugin;

impl Plugin for ExecutableFilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, run_local_file_dialog);
    }
}

fn run_local_file_dialog(
    mut file_dialog: ResMut<ShowFileDialog>,
    mut contexts: EguiContexts,
    turtles: Query<&TurtleInstance>,
    active_turtle_res: Res<ActiveTurtleRes>,
    mut ws_writer: EventWriter<C2SPacket>,
) {
    let curr_turtle = turtles
        .iter()
        .filter(|t| t.turtle.is_online)
        .map(|t| (*t).clone())
        .find(|t| t.index == active_turtle_res.0);
    file_dialog.show &= curr_turtle.is_some();
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
                            ws_writer.send(C2SPacket::SendLuaToTurtle {
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
