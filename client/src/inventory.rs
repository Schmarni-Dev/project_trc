use actually_usable_voxel_mesh_gen::util::string_to_color;
use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, Ui};
use custom_egui_widgets::item_box::{item_box, SlotActionProvider};

#[derive(Clone)]
pub struct Item {
    pub id: String,
    pub count: usize,
    pub name: String,
    pub enchantments: Vec<Enchantment>,
}

#[derive(Clone)]
pub struct Enchantment {
    pub id: String,
    pub level: u8,
}

#[derive(Clone, Component)]
pub struct Inventory {
    pub items: Vec<Option<Item>>,
    pub slots: usize,
}
#[derive(Clone, Copy, Component, Deref, DerefMut)]
pub struct ShowInventoryWindow(pub bool);

#[derive(Clone, Component)]
pub struct InventoryWindowName(pub String);

pub fn draw_inventory_grid<T: SlotActionProvider>(
    ui: &mut Ui,
    provider: &mut T,
    inv: &Inventory,
    selected: Option<usize>,
    max_items_per_row: Option<usize>,
) {
    egui::Grid::new("inv_grid")
        .spacing(egui::Vec2::splat(2.0))
        .show(ui, |ui| {
            for (item, slot) in
                (0..inv.slots).map(|i| (inv.items.get(i).and_then(|it| it.as_ref()), i))
            {
                ui.add(draw_item_box(item, slot, provider, selected));
                match max_items_per_row {
                    Some(max) if slot % max == max - 1 => {
                        ui.end_row();
                    }
                    _ => {}
                }
            }
        });
}

pub fn draw_item_box<'a, T: SlotActionProvider>(
    item: Option<&'a Item>,
    slot_id: usize,
    provider: &'a mut T,
    selected: Option<usize>,
) -> impl egui::Widget + 'a {
    let color = match item {
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
        item.map_or(0, |i| i.count),
        item.map_or("", |i| &i.name),
        color,
        0.9,
        slot_id,
        provider,
        selected.is_some_and(|s| s == slot_id),
    )
}
