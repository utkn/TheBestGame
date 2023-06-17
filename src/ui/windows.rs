use notan::egui;

use crate::core::primitive_components::Position;
use crate::core::{EntityRef, State, StateCommands};
use crate::item::ItemLocation;

use super::widgets::*;
use super::UiState;

#[derive(Clone, Copy, Debug)]
pub enum WindowType {
    Storage(EntityRef),
    Equipment(EntityRef),
}

impl From<Option<WindowType>> for ItemLocation {
    fn from(opt_win_type: Option<WindowType>) -> Self {
        match opt_win_type {
            Some(WindowType::Storage(entity)) => ItemLocation::Storage(entity),
            Some(WindowType::Equipment(entity)) => ItemLocation::Equipment(entity),
            None => ItemLocation::Ground,
        }
    }
}

pub trait Window {
    fn window_id(&self) -> egui::Id;
    fn window_type(&self) -> WindowType;
    fn requested_pos(&self) -> (f32, f32);
    fn size(&self) -> (f32, f32);
    fn add_into(
        &mut self,
        ctx: &egui::Context,
        pos: (f32, f32),
        ui_cmds: &mut StateCommands,
        ui_state: &mut UiState,
    );
}

pub(super) struct EquipmentWindow<'a>(pub(super) EntityRef, pub(super) &'a State);

impl<'a> Window for EquipmentWindow<'a> {
    fn window_id(&self) -> egui::Id {
        format!("EquipmentWindow[{:?}]", self.0).into()
    }

    fn window_type(&self) -> WindowType {
        WindowType::Equipment(self.0)
    }

    fn add_into(
        &mut self,
        ctx: &egui::Context,
        pos: (f32, f32),
        ui_cmds: &mut StateCommands,
        ui_state: &mut UiState,
    ) {
        egui::Window::new("Equipment")
            .id(self.window_id())
            .fixed_size(self.size())
            .title_bar(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());
                ui.add(EquipmentWidget(&self.0, self.1, ui_cmds, ui_state));
            });
    }

    fn requested_pos(&self) -> (f32, f32) {
        self.1
            .select_one::<(Position,)>(&self.0)
            .map(|(pos,)| (pos.x, pos.y))
            .unwrap_or_default()
    }

    fn size(&self) -> (f32, f32) {
        (200., 100.)
    }
}

pub(super) struct StorageWindow<'a>(pub(super) EntityRef, pub(super) &'a State);

impl<'a> Window for StorageWindow<'a> {
    fn window_id(&self) -> egui::Id {
        format!("StorageWindow[{:?}]", self.0).into()
    }

    fn window_type(&self) -> WindowType {
        WindowType::Storage(self.0)
    }

    fn add_into(
        &mut self,
        ctx: &egui::Context,
        pos: (f32, f32),
        ui_cmds: &mut StateCommands,
        ui_state: &mut UiState,
    ) {
        egui::Window::new("Storage")
            .id(self.window_id())
            .fixed_size(self.size())
            .title_bar(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());
                ui.add(StorageWidget(&self.0, self.1, ui_cmds, ui_state))
            });
    }

    fn requested_pos(&self) -> (f32, f32) {
        self.1
            .select_one::<(Position,)>(&self.0)
            .map(|(pos,)| (pos.x, pos.y))
            .unwrap_or_default()
    }

    fn size(&self) -> (f32, f32) {
        (100., 100.)
    }
}
