use std::collections::HashMap;

use itertools::Itertools;
use notan::egui;

use crate::{
    core::{EntityRef, EntityRefBag, Name, State, StateCommands},
    interaction::InteractTarget,
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
    windows: Vec<Box<dyn Window + 'a>>,
}

impl<'a> UiBuilder<'a> {
    fn add_window<T: Window + 'a>(&mut self, win: T) {
        self.windows.push(Box::new(win));
    }

    fn build(&mut self, game_state: &'a State) -> HashMap<egui::Id, WindowType> {
        let player_entity = EntityRef::new(0, 0);
        self.add_window(NeedsWindow(player_entity));
        self.add_window(EquipmentWindow(player_entity));
        self.add_window(StorageWindow {
            title: "Backpack",
            storage_entity: player_entity,
        });
        let active_storages = game_state
            .select::<(Storage, InteractTarget<Storage>)>()
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
        game_state: &'a State,
        ui_cmds: &mut StateCommands,
        ui_state: &mut UiState,
    ) -> HashMap<egui::Id, WindowType> {
        let window_types = self.build(game_state);
        self.windows.into_iter().for_each(|mut w| {
            w.add_into(ctx, game_state, ui_cmds, ui_state);
        });
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
