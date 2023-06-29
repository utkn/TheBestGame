use std::collections::HashMap;

use itertools::Itertools;
use notan::egui;

use crate::{
    character::CharacterBundle,
    item::{Equipment, ItemTransferReq, Storage},
    prelude::*,
};

mod ui_state;
mod widgets;
mod windows;

pub use ui_state::UiState;
use windows::*;

pub struct UiBuilder<'a, R> {
    windows: Vec<Box<dyn Window<R> + 'a>>,
}

impl<'a, R> Default for UiBuilder<'a, R> {
    fn default() -> Self {
        Self {
            windows: Default::default(),
        }
    }
}

impl<'a, R: StateReader> UiBuilder<'a, R> {
    fn add_window<W: Window<R> + 'a>(&mut self, win: W) {
        self.windows.push(Box::new(win));
    }

    fn build(&mut self, game_state: &'a R) -> HashMap<egui::Id, WindowType> {
        let player_entity = EntityRef::new(0, 0);
        let player_char = game_state
            .read_bundle::<CharacterBundle>(&player_entity)
            .unwrap();
        self.add_window(NeedsWindow(player_entity));
        self.add_window(EquipmentWindow {
            title: "Equipment",
            equipment_entity: player_entity,
            is_player_equipment: true,
        });
        if let Some(character_backpack) = player_char.get_backpack(game_state) {
            self.add_window(StorageWindow {
                title: "Backpack",
                storage_entity: *character_backpack,
                is_player_storage: true,
            });
        }
        let active_storages = game_state
            .select::<(Storage, InteractTarget<Storage>)>()
            .filter(|(_, (_, intr))| intr.actors.contains(&player_entity))
            .collect_vec();
        let active_equipments = game_state
            .select::<(Equipment, InteractTarget<Equipment>)>()
            .filter(|(_, (_, intr))| intr.actors.contains(&player_entity))
            .collect_vec();
        for (storage_entity, _) in active_storages {
            let storage_name = game_state
                .select_one::<(Name,)>(&storage_entity)
                .map(|(name,)| name.0)
                .unwrap_or("unnamed");
            self.add_window(StorageWindow {
                title: storage_name,
                storage_entity,
                is_player_storage: false,
            });
        }
        for (equipment_entity, _) in active_equipments {
            let storage_name = game_state
                .select_one::<(Name,)>(&equipment_entity)
                .map(|(name,)| name.0)
                .unwrap_or("unnamed");
            self.add_window(EquipmentWindow {
                title: storage_name,
                equipment_entity,
                is_player_equipment: false,
            });
        }
        self.windows
            .iter()
            .map(|win| (win.window_id(), win.window_type()))
            .collect()
    }

    pub fn build_and_draw(
        mut self,
        ctx: &egui::Context,
        game_state: &'a R,
        ui_state: &mut UiState,
    ) -> HashMap<egui::Id, WindowType> {
        let window_types = self.build(game_state);
        self.windows.into_iter().for_each(|mut w| {
            w.add_into(ctx, game_state, ui_state);
        });
        window_types
    }
}

pub fn draw_ui<R: StateReader>(
    ctx: &egui::Context,
    game_state: &R,
    ui_cmds: &mut StateCommands,
    ui_state: &mut UiState,
) {
    let window_types = UiBuilder::<R>::default().build_and_draw(ctx, game_state, ui_state);
    if let Some(drag_result) = ui_state.item_drag.try_complete(ctx) {
        let from_win_type = drag_result
            .from_win_id
            .and_then(|id| window_types.get(&id))
            .cloned();
        let to_win_type = drag_result
            .to_win_id
            .and_then(|id| window_types.get(&id))
            .cloned();
        drag_result
            .dragged_item_stack
            .into_iter()
            .for_each(|item_entity| {
                let item_transfer_req = ItemTransferReq {
                    item_entity,
                    from_loc: from_win_type.into(),
                    to_loc: to_win_type.into(),
                };
                ui_cmds.emit_event(item_transfer_req);
            });
    }
}
