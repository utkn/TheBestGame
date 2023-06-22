use notan::egui;

use crate::item::ItemStack;

#[derive(Clone, Debug)]
pub(super) struct DragResult {
    pub(super) from_win_id: Option<egui::Id>,
    pub(super) to_win_id: Option<egui::Id>,
    pub(super) dragged_item_stack: ItemStack,
}

#[derive(Clone, Default, Debug)]
pub struct ItemDragState {
    from_position: Option<(f32, f32)>,
    to_position: Option<(f32, f32)>,
    dragging_item_stack: Option<ItemStack>,
}

impl ItemDragState {
    pub fn is_dragging(&self) -> bool {
        self.dragging_item_stack.is_some()
    }

    pub(super) fn start(&mut self, stack: ItemStack, pos: (f32, f32)) {
        self.from_position = Some(pos);
        self.dragging_item_stack = Some(stack);
    }

    pub(super) fn stop(&mut self, pos: (f32, f32)) {
        if let Some(_) = self.from_position {
            self.to_position = Some(pos);
        }
    }

    pub(super) fn try_complete(&mut self, ctx: &egui::Context) -> Option<DragResult> {
        let from_pos = self.from_position?;
        let to_pos = self.to_position?;
        let dragged_item_stack = self.dragging_item_stack.take()?;
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
        Some(DragResult {
            dragged_item_stack,
            from_win_id,
            to_win_id,
        })
    }
}

#[derive(Clone, Default, Debug)]
pub struct UiState {
    pub item_drag: ItemDragState,
}
