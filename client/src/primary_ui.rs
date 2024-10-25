use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Ui},
    EguiContexts,
};

use crate::turtle::{PrimaryTurtle, Turtle, TurtleIndex, TurtleIsOnline};

pub struct PrimaryUiPlugin;
impl Plugin for PrimaryUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_primary_ui);
    }
}

pub fn draw_primary_ui(
    online_turtles: Query<(Entity, &TurtleIndex, &Name), (With<Turtle>, With<TurtleIsOnline>)>,
    mut primary_turtle: ResMut<PrimaryTurtle>,
    mut contexts: EguiContexts,
) {
    egui::TopBottomPanel::top("TRC").show(contexts.ctx_mut(), move |ui| {
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                let mut prim_turtle = primary_turtle.0;
                let mut turtle_dropdown = egui::ComboBox::from_label("Turtle");
                if let Ok((_, index, name)) = online_turtles.get(primary_turtle.0) {
                    turtle_dropdown =
                        turtle_dropdown.selected_text(format!("{}: {}", **index, &**name));
                }

                turtle_dropdown.show_ui(ui, |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.set_min_width(60.0);
                    for (e, index, name) in &online_turtles {
                        ui.selectable_value(
                            &mut prim_turtle,
                            e,
                            &format!("{}: {}", **index, &**name),
                        );
                    }
                });
                if prim_turtle != primary_turtle.0 {
                    primary_turtle.0 = prim_turtle;
                }
            })
        })
    });
}
