use std::{rc::Rc, sync::mpsc};

use egui::{Color32, FontId, Response, RichText, Slider, Stroke, Ui};

pub enum ItemSlotActions {
    SelectSlot,
    Transfer(u8),
    Refuel,
}

pub type TX = mpsc::Sender<(u8, ItemSlotActions)>;

pub fn item_box<'a>(
    amount: u8,
    name: Rc<str>,
    color: Color32,
    scale: f32,
    slot_id: u8,
    tx: TX,
    selected: bool,
    amount_modifier: &'a mut u8,
) -> impl egui::Widget + 'a {
    move |ui: &mut Ui| {
        item_box_render(
            ui,
            amount,
            name,
            color,
            scale,
            slot_id,
            tx,
            selected,
            amount_modifier,
        )
    }
}

fn get_gray_scale(color: &Color32) -> f32 {
    let r = color.r() as f32 / u8::MAX as f32;
    let g = color.g() as f32 / u8::MAX as f32;
    let b = color.b() as f32 / u8::MAX as f32;
    let luma = (0.2126 * r) + (0.7152 * g) + (0.0722 * b);
    return luma;
}

fn item_box_context_menu<'a>(
    slot_id: u8,
    amount: u8,
    name: Rc<str>,
    tx: TX,
    amount_modifier: &'a mut u8,
) -> impl FnOnce(&mut Ui) + 'a {
    move |ui: &mut Ui| item_box_context_menu_render(ui, name, amount, slot_id, tx, amount_modifier)
}

fn item_box_render(
    ui: &mut Ui,
    amount: u8,
    name: Rc<str>,
    color: Color32,
    scale: f32,
    slot_id: u8,
    tx: TX,
    selected: bool,
    amount_modifier: &mut u8,
) -> Response {
    let desired_size = egui::Vec2::splat(2.0 * scale * ui.spacing().interact_size.y);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    let luma = get_gray_scale(&color);
    let inver = 1.0 - luma.clamp(0.0, 1.0);
    let gray_color = Color32::from_gray(((255u8 as f32 * inver) as i64).try_into().unwrap());

    response = response
        .context_menu(item_box_context_menu(
            slot_id,
            amount,
            name.clone(),
            tx.clone(),
            amount_modifier,
        ))
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    if amount != 0 {
        response = response
            .on_hover_text_at_pointer(RichText::new(name.as_ref()).strong().monospace().heading());
    }

    if response.clicked() {
        tx.send((slot_id, ItemSlotActions::SelectSlot)).unwrap();
    }

    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, 8.0 * scale, color.clone());
        if selected {
            ui.painter().rect_stroke(
                rect,
                8.0 * scale,
                Stroke::new(3.0 * scale, Color32::LIGHT_GRAY),
            );
        }
        if amount != 0 {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                amount.to_string(),
                FontId::monospace(24.0 * scale),
                gray_color,
            );
        }
    }
    response
}

fn item_box_context_menu_render(
    ui: &mut Ui,
    name: Rc<str>,
    amount: u8,
    slot_id: u8,
    tx: TX,
    amount_modifier: &mut u8,
) {
    if amount != 0 {
        ui.label(RichText::new(name.as_ref()).strong().heading());
    }
    ui.add(
        Slider::new(amount_modifier, 0..=64)
            .integer()
            .text("Amount"),
    );
    if ui.button("Select Slot").clicked() {
        tx.send((slot_id, ItemSlotActions::SelectSlot)).unwrap();
    }

    if ui.button("Move the Selected Stack here").clicked() {
        tx.send((slot_id, ItemSlotActions::Transfer(64))).unwrap();
    }
    if ui
        .button(format!(
            "Move {} Items from the Selected Stack here",
            amount_modifier
        ))
        .clicked()
    {
        tx.send((slot_id, ItemSlotActions::Transfer(*amount_modifier)))
            .unwrap();
    }
    if ui.button("Refuel using the Selected Slot").clicked() {
        tx.send((slot_id,ItemSlotActions::Refuel)).unwrap()
    }
    // if ui.button("Move Half of the Selected Stack here").clicked() {
    //     tx.send((slot_id, ItemSlotActions::Transfer(amount / 2)))
    //         .unwrap();
    // }
    // if ui
    //     .button("Move 16 items from the Selected Stack here")
    //     .clicked()
    // {
    //     tx.send((slot_id, ItemSlotActions::Transfer(amount.min(16))))
    //         .unwrap();
    // }
    // if ui
    //     .button("Move 32 items from the Selected Stack here")
    //     .clicked()
    // {
    //     tx.send((slot_id, ItemSlotActions::Transfer(amount.min(32))))
    //         .unwrap();
    // }
    // if ui
    //     .button("Move 48 items from the Selected Stack here")
    //     .clicked()
    // {
    //     tx.send((slot_id, ItemSlotActions::Transfer(amount.min(48))))
    //         .unwrap();
    // }
    // if ui.button("Move 3/4 of the Selected Stack here").clicked() {
    //     tx.send((
    //         slot_id,
    //         ItemSlotActions::Transfer(((amount as f32 / 1.5).floor() as i64).try_into().unwrap()),
    //     ))
    //     .unwrap();
    // }
    // if ui.button("Move 1/4 of the Selected Stack here").clicked() {
    //     tx.send((slot_id, ItemSlotActions::Transfer(amount / 4)))
    //         .unwrap();
    // }
}
