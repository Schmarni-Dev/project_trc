use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Id},
    EguiContexts,
};
use common::turtle::Inventory;

use crate::systems::ActiveTurtle;

pub struct ExternalInvSupportPlugin;

impl Plugin for ExternalInvSupportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui);
    }
}

fn ui(mut contexts: EguiContexts, query: Query<&ConnectedInventories, With<ActiveTurtle>>) {
    for invs in &query {
        for inv in &invs.inventories {
            let mut hasher = DefaultHasher::default();

            inv.ident.hash(&mut hasher);

            egui::Window::new(&inv.name)
                .id(Id::new(hasher.finish()))
                .show(contexts.ctx_mut(), |ui| {
                    ui.label("inv_ui");
                });
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct ConnectedInventories {
    inventories: Vec<ConnectedInventory>,
}

#[derive(Clone, Debug, Reflect)]
pub struct ConnectedInventory {
    ident: String,
    name: String,
    inv: Inventory,
}
