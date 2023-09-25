#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::mpsc;

use custom_egui_widgets::{item_box::item_box, CircleDisplay};
use eframe::egui;
use egui::Color32;
fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2 { x: 1280., y: 720. }),
        ..Default::default()
    };

    _ = eframe::run_native(
        "egui test app",
        options,
        Box::new(|_cc| Box::<App>::default()),
    );
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (tx, rx) = mpsc::channel();
        // ctx.set_pixels_per_point(5.);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.add(egui::Slider::new(&mut self.value, 0..=self.max_value).text("Value"));
            if ui.button("Click each year").clicked() {
                //     self.age += 1;
            }
            ui.label(format!(
                " Value {}, Max Value {}",
                self.value, self.max_value
            ));
            ui.add(item_box(
                16,
                "minecraft:moss_block".into(),
                Color32::DARK_RED,
                2.0,
                1,
                tx.clone(),
                true,
            ));
            ui.add(
                CircleDisplay::new()
                    .render_background()
                    .stroke_color(Color32::KHAKI)
                    .size(2.5)
                    .build(&self.value, &self.max_value),
            )
        });
        while let Ok((id, action)) = rx.try_recv() {
            match action {
                custom_egui_widgets::item_box::ItemSlotActions::SelectSlot => {
                    println!("Select Slot: {id}")
                }
                custom_egui_widgets::item_box::ItemSlotActions::Transfer(amount) => {
                    println!("Transfer {amount} items to Slot: {id}")
                }
            }
        }
    }
}

struct App {
    value: i32,
    max_value: i32,
}
impl Default for App {
    fn default() -> Self {
        Self {
            value: 0,
            max_value: 512,
        }
    }
}
