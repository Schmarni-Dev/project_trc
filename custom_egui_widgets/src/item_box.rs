use derive_more::{Deref, DerefMut};
use egui::{Color32, FontId, Response, RichText, Stroke, Ui};
use std::{borrow::Cow, sync::mpsc};

pub enum SlotInteraction<T> {
    SlotClicked,
    Action(T),
}

pub trait SlotAction: Sized + Clone + 'static {
    fn get_name(&self) -> Cow<'static, str>;
}

pub trait SlotActionProvider: Sized + 'static {
    type Action: SlotAction;
    fn get_secondary_actions(&self, slot_id: usize) -> Vec<Self::Action>;
    fn get_primary_action(&self) -> Self::Action;
    fn interact(&mut self, slot: usize, action: Self::Action);
    fn draw_ctx_ui(&mut self, ui: &mut Ui);
}

#[derive(Clone, Deref, DerefMut)]
pub struct SlotActionSender<T: SlotActionProvider>(mpsc::Sender<(usize, SlotInteraction<T>)>);

pub fn item_box<'a, T: SlotActionProvider>(
    amount: usize,
    name: &'a str,
    color: Color32,
    scale: f32,
    slot_id: usize,
    provider: &'a mut T,
    selected: bool,
) -> impl egui::Widget + 'a {
    move |ui: &mut Ui| item_box_render(ui, amount, name, color, scale, slot_id, provider, selected)
}

fn get_gray_scale(color: &Color32) -> f32 {
    let r = color.r() as f32 / u8::MAX as f32;
    let g = color.g() as f32 / u8::MAX as f32;
    let b = color.b() as f32 / u8::MAX as f32;
    let luma = (0.2126 * r) + (0.7152 * g) + (0.0722 * b);
    return luma;
}

fn item_box_context_menu<T: SlotActionProvider>(
    slot_id: usize,
    provider: &mut T,
) -> impl FnOnce(&mut Ui) + '_ {
    move |ui: &mut Ui| item_box_context_menu_render(ui, slot_id, provider)
}

#[allow(clippy::too_many_arguments)]
fn item_box_render<T: SlotActionProvider>(
    ui: &mut Ui,
    amount: usize,
    name: &str,
    color: Color32,
    scale: f32,
    slot_id: usize,
    provider: &mut T,
    selected: bool,
) -> Response {
    let desired_size = egui::Vec2::splat(2.0 * scale * ui.spacing().interact_size.y);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    let luma = get_gray_scale(&color);
    let inver = 1.0 - luma.clamp(0.0, 1.0);
    let gray_color = Color32::from_gray(((255f32 * inver) as i64).try_into().unwrap());

    response.context_menu(item_box_context_menu(slot_id, provider));
    // .on_hover_cursor(egui::CursorIcon::PointingHand);
    response =
        response.on_hover_text_at_pointer(RichText::new(name).strong().monospace().heading());

    if response.clicked() {
        provider.interact(slot_id, provider.get_primary_action());
    }

    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, 8.0 * scale, color);
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

fn item_box_context_menu_render<T: SlotActionProvider>(
    ui: &mut Ui,
    slot_id: usize,
    provider: &mut T,
) {
    for action in provider.get_secondary_actions(slot_id).into_iter() {
        if ui.button(action.get_name()).clicked() {
            provider.interact(slot_id, action.clone());
        }
    }
}
