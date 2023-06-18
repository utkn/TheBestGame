use notan::egui;

use crate::core::EntityRef;

#[derive(Clone, Debug)]
pub(super) struct DragResult {
    pub(super) from_win_id: Option<egui::Id>,
    pub(super) to_win_id: Option<egui::Id>,
    pub(super) dragged_item: EntityRef,
}

#[derive(Clone, Default, Debug)]
pub struct ItemDragState {
    from_position: Option<(f32, f32)>,
    to_position: Option<(f32, f32)>,
    dragging_item: Option<EntityRef>,
}

impl ItemDragState {
    pub fn is_dragging(&self) -> bool {
        self.dragging_item.is_some()
    }

    pub(super) fn dragging(&self) -> Option<&EntityRef> {
        self.dragging_item.as_ref()
    }

    pub(super) fn start(&mut self, item: EntityRef, pos: (f32, f32)) {
        self.from_position = Some(pos);
        self.dragging_item = Some(item);
    }

    pub(super) fn stop(&mut self, pos: (f32, f32)) {
        if let Some(_) = self.from_position {
            self.to_position = Some(pos);
        }
    }

    pub(super) fn try_complete(&mut self, ctx: &egui::Context) -> Option<DragResult> {
        let dragged_item = self.dragging_item?;
        let from_pos = self.from_position?;
        let to_pos = self.to_position?;
        let from_win_id = ctx
            .layer_id_at(egui::Pos2 {
                x: from_pos.0,
                y: from_pos.1,
            })
            .map(|layer| layer.id);
        let to_win_id = ctx
            .layer_id_at(egui::Pos2 {
                x: to_pos.0,
                y: to_pos.1,
            })
            .map(|layer| layer.id);
        self.from_position.take();
        self.to_position.take();
        self.dragging_item.take();
        Some(DragResult {
            dragged_item,
            from_win_id,
            to_win_id,
        })
    }
}

#[derive(Clone, Default, Debug)]
pub struct UiState {
    pub item_drag: ItemDragState,
}
