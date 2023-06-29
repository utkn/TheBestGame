use notan::egui;

use crate::camera::map_to_screen_cords;
use crate::item::ItemLocation;
use crate::prelude::*;

use super::widgets::*;
use super::UiState;

const WINDOW_WIDTH: f32 = 165.;

#[derive(Clone, Copy, Debug)]
pub enum WindowType {
    Storage(EntityRef),
    Equipment(EntityRef),
    Needs(EntityRef),
}

impl From<Option<WindowType>> for ItemLocation {
    fn from(opt_win_type: Option<WindowType>) -> Self {
        match opt_win_type {
            Some(WindowType::Storage(entity)) => ItemLocation::Storage(entity),
            Some(WindowType::Equipment(entity)) => ItemLocation::Equipment(entity),
            _ => ItemLocation::Ground,
        }
    }
}

pub trait Window<R: StateReader> {
    fn window_id(&self) -> egui::Id;
    fn window_type(&self) -> WindowType;
    fn add_into(&mut self, ctx: &egui::Context, game_state: &R, ui_state: &mut UiState);
}

pub(super) struct EquipmentWindow {
    pub(super) title: &'static str,
    pub(super) equipment_entity: EntityRef,
    pub(super) is_player_equipment: bool,
}

impl<R: StateReader> Window<R> for EquipmentWindow {
    fn window_id(&self) -> egui::Id {
        format!("EquipmentWindow[{:?}]", self.equipment_entity).into()
    }

    fn window_type(&self) -> WindowType {
        WindowType::Equipment(self.equipment_entity)
    }

    fn add_into(&mut self, ctx: &egui::Context, game_state: &R, ui_state: &mut UiState) {
        let screen_width = ctx.input().screen_rect().width();
        let screen_height = ctx.input().screen_rect().height();
        let mut win = egui::Window::new(self.title)
            .id(Window::<R>::window_id(self))
            .collapsible(false)
            .default_width(WINDOW_WIDTH)
            .resizable(false);
        // Handle alignment & positioning.
        if self.is_player_equipment {
            win = win.anchor(egui::Align2::RIGHT_TOP, (-10., 130.));
        } else {
            let (x, y) = game_state
                .select_one::<(Transform,)>(&self.equipment_entity)
                .map(|(pos,)| (pos.x, pos.y))
                .map(|(x, y)| map_to_screen_cords(x, y, screen_width, screen_height, game_state))
                .unwrap_or_default();
            win = win.current_pos((x, y + 10.)).pivot(egui::Align2::RIGHT_TOP);
        }
        win.show(ctx, |ui| {
            ui.set_width(ui.available_width());
            ui.add(EquipmentWidget(
                &self.equipment_entity,
                game_state,
                ui_state,
            ));
        });
    }
}

pub(super) struct StorageWindow {
    pub(super) title: &'static str,
    pub(super) storage_entity: EntityRef,
    pub(super) is_player_storage: bool,
}

impl<R: StateReader> Window<R> for StorageWindow {
    fn window_id(&self) -> egui::Id {
        format!("StorageWindow[{:?}]", self.storage_entity).into()
    }

    fn window_type(&self) -> WindowType {
        WindowType::Storage(self.storage_entity)
    }

    fn add_into(&mut self, ctx: &egui::Context, game_state: &R, ui_state: &mut UiState) {
        let screen_width = ctx.input().screen_rect().width();
        let screen_height = ctx.input().screen_rect().height();
        let mut win = egui::Window::new(self.title)
            .id(Window::<R>::window_id(self))
            .collapsible(false)
            .default_width(WINDOW_WIDTH)
            .default_height(150.)
            .vscroll(true)
            .resizable(false);
        // Handle alignment & positioning.
        if self.is_player_storage {
            win = win.anchor(egui::Align2::RIGHT_TOP, (-10., 405.));
        } else {
            let (x, y) = game_state
                .select_one::<(Transform,)>(&self.storage_entity)
                .map(|(pos,)| (pos.x, pos.y))
                .map(|(x, y)| map_to_screen_cords(x, y, screen_width, screen_height, game_state))
                .unwrap_or_default();
            win = win.current_pos((x, y + 10.));
        }
        win.show(ctx, |ui| {
            ui.set_width(ui.available_width());
            ui.add(StorageWidget(&self.storage_entity, game_state, ui_state))
        });
    }
}

pub(super) struct NeedsWindow(pub(super) EntityRef);

impl<R: StateReader> Window<R> for NeedsWindow {
    fn window_id(&self) -> egui::Id {
        format!("NeedsWindow[{:?}]", self.0).into()
    }

    fn window_type(&self) -> WindowType {
        WindowType::Needs(self.0)
    }

    fn add_into(&mut self, ctx: &egui::Context, game_state: &R, ui_state: &mut UiState) {
        egui::Window::new("Needs")
            .id(Window::<R>::window_id(self))
            .anchor(egui::Align2::RIGHT_TOP, (-10., 10.))
            .collapsible(false)
            .title_bar(false)
            .default_width(WINDOW_WIDTH)
            .default_height(90.)
            // .frame(egui::Frame::none())
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());
                ui.add(NeedsWidget(&self.0, game_state, ui_state));
            });
    }
}
