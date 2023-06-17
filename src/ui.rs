use std::collections::HashMap;

use itertools::Itertools;
use notan::egui;

use crate::{
    core::{EntityRef, EntityRefStorage, State, StateCommands},
    interaction::Interactable,
    item::ItemTransferReq,
    storage::Storage,
};

mod ui_state;
mod widgets;
mod windows;

pub use ui_state::UiState;
use windows::*;

#[derive(Default)]
pub struct UiBuilder<'a> {
    windows: HashMap<egui::Id, Box<dyn Window + 'a>>,
    calculated_pos: HashMap<egui::Id, (f32, f32)>,
}

impl<'a> UiBuilder<'a> {
    fn add_window<T: Window + 'a>(&mut self, win: T) {
        self.windows.insert(win.window_id(), Box::new(win));
    }

    fn build(&mut self, game_state: &'a State) -> HashMap<egui::Id, WindowType> {
        let player_entity = EntityRef::new(0, 0);
        self.add_window(EquipmentWindow(player_entity, game_state));
        self.add_window(StorageWindow(player_entity, game_state));
        let active_storages = game_state
            .select::<(Interactable, Storage)>()
            .filter(|(_, (intr, _))| intr.actors.contains(&player_entity))
            .collect_vec();
        for (storage_entity, (_, _)) in active_storages {
            self.add_window(StorageWindow(storage_entity, game_state));
        }
        self.windows
            .iter()
            .map(|(win_id, win)| (*win_id, win.window_type()))
            .collect()
    }

    fn layout(&mut self) {
        // TODO: perform custom layouting of the windows
        let requested_bounds = self
            .windows
            .iter()
            .map(|(k, v)| {
                let (x, y) = v.requested_pos();
                let (w, h) = v.size();
                (*k, sepax2d::aabb::AABB::new((x, y), w, h))
            })
            .sorted_by_key(|(k, _)| format!("{:?}", k))
            .collect_vec();
        requested_bounds.into_iter().for_each(|(k, v)| {
            self.calculated_pos.insert(k, v.position);
        });
    }

    fn draw(self, ctx: &egui::Context, ui_cmds: &'a mut StateCommands, ui_state: &'a mut UiState) {
        self.windows.into_iter().for_each(|(id, mut w)| {
            let effective_pos = self.calculated_pos.get(&id).cloned().unwrap();
            w.add_into(ctx, effective_pos, ui_cmds, ui_state);
        })
    }

    pub fn build_and_draw(
        mut self,
        ctx: &egui::Context,
        game_state: &'a State,
        ui_cmds: &mut StateCommands,
        ui_state: &mut UiState,
    ) -> HashMap<egui::Id, WindowType> {
        let window_types = self.build(game_state);
        self.layout();
        self.draw(ctx, ui_cmds, ui_state);
        window_types
    }
}

pub fn draw_ui(
    ctx: &egui::Context,
    game_state: &State,
    ui_cmds: &mut StateCommands,
    ui_state: &mut UiState,
) {
    let window_types = UiBuilder::default().build_and_draw(ctx, game_state, ui_cmds, ui_state);
    if let Some(drag_result) = ui_state.item_drag.try_complete(ctx) {
        let item_entity = drag_result.dragged_item;
        let from_win_type = drag_result
            .from_win_id
            .and_then(|id| window_types.get(&id))
            .cloned();
        let to_win_type = drag_result
            .to_win_id
            .and_then(|id| window_types.get(&id))
            .cloned();
        let item_transfer_req = ItemTransferReq {
            item_entity,
            from_loc: from_win_type.into(),
            to_loc: to_win_type.into(),
        };
        ui_cmds.emit_event(item_transfer_req);
    }
}
