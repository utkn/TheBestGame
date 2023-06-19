use itertools::Itertools;
use notan::egui;

use crate::activation::Activatable;
use crate::core::*;
use crate::entity_insights::EntityLocation;
use crate::interaction::Interactable;
use crate::storage::Storage;

use super::widgets::*;
use super::UiState;

#[derive(Clone, Copy, Debug)]
pub enum WindowType {
    Storage(EntityRef),
    Equipment(EntityRef),
    Needs(EntityRef),
}

impl From<Option<WindowType>> for EntityLocation {
    fn from(opt_win_type: Option<WindowType>) -> Self {
        match opt_win_type {
            Some(WindowType::Storage(entity)) => EntityLocation::Storage(entity),
            Some(WindowType::Equipment(entity)) => EntityLocation::Equipment(entity),
            _ => EntityLocation::Ground,
        }
    }
}

pub trait Window {
    fn window_id(&self) -> egui::Id;
    fn window_type(&self) -> WindowType;
    fn add_into(
        &mut self,
        ctx: &egui::Context,
        game_state: &State,
        ui_cmds: &mut StateCommands,
        ui_state: &mut UiState,
    );
}

pub(super) struct EquipmentWindow(pub(super) EntityRef);

impl Window for EquipmentWindow {
    fn window_id(&self) -> egui::Id {
        format!("EquipmentWindow[{:?}]", self.0).into()
    }

    fn window_type(&self) -> WindowType {
        WindowType::Equipment(self.0)
    }

    fn add_into(
        &mut self,
        ctx: &egui::Context,
        game_state: &State,
        ui_cmds: &mut StateCommands,
        ui_state: &mut UiState,
    ) {
        egui::Window::new("Equipment")
            .id(self.window_id())
            .anchor(egui::Align2::RIGHT_TOP, (-10., 120.))
            .collapsible(false)
            .title_bar(false)
            .fixed_size((140., 90.))
            .resizable(false)
            // .frame(egui::Frame::none())
            .show(ctx, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());
                ui.add(EquipmentWidget(&self.0, game_state, ui_cmds, ui_state));
            });
    }
}

pub(super) struct StorageWindow {
    pub(super) title: &'static str,
    pub(super) storage_entity: EntityRef,
}

impl Window for StorageWindow {
    fn window_id(&self) -> egui::Id {
        format!("StorageWindow[{:?}]", self.storage_entity).into()
    }

    fn window_type(&self) -> WindowType {
        WindowType::Storage(self.storage_entity)
    }

    fn add_into(
        &mut self,
        ctx: &egui::Context,
        game_state: &State,
        ui_cmds: &mut StateCommands,
        ui_state: &mut UiState,
    ) {
        let window_width = 140.;
        let is_player_storage = game_state
            .select_one::<(Controller,)>(&self.storage_entity)
            .is_some();
        let mut win = egui::Window::new(self.title)
            .id(self.window_id())
            .collapsible(false)
            .default_width(window_width)
            .resizable(false);
        // Get the active storages, i.e., the storages that are being interacted by this storage.
        let active_storages = game_state
            .select::<(Interactable, Storage, Transform, Activatable<Storage>)>()
            .filter(|(_, (intr, _, _, activatable))| {
                intr.actors.contains(&self.storage_entity) && activatable.curr_state
            })
            .collect_vec();
        // Calculate the position through them.
        let position_with_active_storage = active_storages
            .into_iter()
            .map(|(_, (_, _, pos, _))| ((pos.x + window_width).ceil() as i32, pos.y))
            .max_by_key(|(x, _)| *x);
        // Handle alignment & positioning.
        if is_player_storage {
            if let Some((x, y)) = position_with_active_storage {
                win = win.fixed_pos((x as f32 + 30., y + 10.));
            } else {
                win = win.anchor(egui::Align2::RIGHT_TOP, (-10., 270.));
            }
        } else {
            let storage_pos = game_state
                .select_one::<(Transform,)>(&self.storage_entity)
                .map(|(pos,)| *pos)
                .unwrap_or_default();
            win = win.fixed_pos((storage_pos.x + 10., storage_pos.y + 10.));
        }
        win.show(ctx, |ui| {
            ui.set_width(ui.available_width());
            ui.add(StorageWidget(
                &self.storage_entity,
                game_state,
                ui_cmds,
                ui_state,
            ))
        });
    }
}

pub(super) struct NeedsWindow(pub(super) EntityRef);

impl Window for NeedsWindow {
    fn window_id(&self) -> egui::Id {
        format!("NeedsWindow[{:?}]", self.0).into()
    }

    fn window_type(&self) -> WindowType {
        WindowType::Needs(self.0)
    }

    fn add_into(
        &mut self,
        ctx: &egui::Context,
        game_state: &State,
        ui_cmds: &mut StateCommands,
        ui_state: &mut UiState,
    ) {
        egui::Window::new("Needs")
            .id(self.window_id())
            .anchor(egui::Align2::RIGHT_TOP, (-10., 10.))
            .collapsible(false)
            .title_bar(false)
            .fixed_size((140., 90.))
            // .frame(egui::Frame::none())
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());
                ui.add(NeedsWidget(&self.0, game_state, ui_cmds, ui_state));
            });
    }
}
