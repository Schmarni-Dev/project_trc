use std::num::NonZeroUsize;

use bevy::prelude::*;
use bevy_egui::egui::{self, Slider};
use custom_egui_widgets::item_box::{SlotAction, SlotActionProvider};

use crate::inventory::{draw_inventory_grid, Inventory, InventoryWindowName, ShowInventoryWindow};

pub struct TurtleInventoryPlugin;
impl Plugin for TurtleInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_ui);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TurtleSlotActionType {
    Select,
    MoveSelectedHere,
    MoveAmountFromSelectedHere,
    Refuel,
    MoveHalfSelectedHere,
}

#[derive(Debug, Clone, Copy)]
pub struct TurtleSlotAction {
    pub action_type: TurtleSlotActionType,
    pub amount: NonZeroUsize,
}

impl SlotAction for TurtleSlotAction {
    fn get_name(&self) -> std::borrow::Cow<'static, str> {
        match self.action_type {
            TurtleSlotActionType::Select => "Select Slot".into(),
            TurtleSlotActionType::MoveSelectedHere => "Move Items from Selected Slot Here".into(),
            TurtleSlotActionType::MoveAmountFromSelectedHere => {
                format!("Move {} Items from Selected Slot Here", self.amount).into()
            }
            TurtleSlotActionType::Refuel => "Refuel With Items from this Slot".into(),
            TurtleSlotActionType::MoveHalfSelectedHere => {
                "Move Half of the Items in the Selected Slot Here".into()
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct TurtleInventoryState {
    selected_slot: u8,
    amount: usize,
    current_action: Option<(usize, TurtleSlotAction)>,
}
impl SlotActionProvider for TurtleInventoryState {
    type Action = TurtleSlotAction;

    fn get_secondary_actions(&self, slot_id: usize) -> Vec<Self::Action> {
        let mut vec = Vec::new();
        let amount =
            NonZeroUsize::new(self.amount).unwrap_or(unsafe { NonZeroUsize::new_unchecked(1) });
        if slot_id == self.selected_slot as usize {
            vec.push(TurtleSlotAction {
                amount,
                action_type: TurtleSlotActionType::Select,
            });
        } else {
            vec.push(TurtleSlotAction {
                amount,
                action_type: TurtleSlotActionType::MoveSelectedHere,
            });
            vec.push(TurtleSlotAction {
                amount,
                action_type: TurtleSlotActionType::MoveAmountFromSelectedHere,
            });
            vec.push(TurtleSlotAction {
                amount,
                action_type: TurtleSlotActionType::MoveHalfSelectedHere,
            });
        }
        vec.push(TurtleSlotAction {
            amount,
            action_type: TurtleSlotActionType::Refuel,
        });
        vec
    }

    fn get_primary_action(&self) -> Self::Action {
        let amount =
            NonZeroUsize::new(self.amount).unwrap_or(unsafe { NonZeroUsize::new_unchecked(1) });
        TurtleSlotAction {
            amount,
            action_type: TurtleSlotActionType::Select,
        }
    }

    fn interact(&mut self, slot: usize, action: Self::Action) {
        self.current_action = Some((slot, action));
    }

    fn draw_ctx_ui(&mut self, ui: &mut egui::Ui) {
        ui.add(Slider::new(&mut self.amount, 1..=64).text("Amount: "));
    }
}

fn draw_ui(
    mut query: Query<(
        &Inventory,
        &InventoryWindowName,
        &mut TurtleInventoryState,
        &mut ShowInventoryWindow,
    )>,
    mut egui_ctxs: bevy_egui::EguiContexts,
) {
    for (inv, inv_id, mut inv_state, mut open) in &mut query {
        inv_state.current_action = None;
        egui::Window::new(inv_id.0.as_str())
            .resizable(false)
            .open(&mut open)
            .show(egui_ctxs.ctx_mut(), |ui| {
                let selected_slot = Some(inv_state.selected_slot as usize);
                ui.add(Slider::new(&mut inv_state.amount, 1..=64).text("Amount: "));
                draw_inventory_grid(ui, &mut *inv_state, inv, selected_slot, Some(4))
            });
    }
}
