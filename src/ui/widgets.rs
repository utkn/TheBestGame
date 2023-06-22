use itertools::Itertools;
use notan::egui;

use crate::{
    item::{Equipment, Item, ItemStack, Storage},
    needs::Needs,
    prelude::*,
};

use super::UiState;

pub(super) struct ItemStackWidget<'a>(
    pub(super) &'a ItemStack,
    pub(super) &'a State,
    pub(super) &'a mut StateCommands,
    pub(super) &'a mut UiState,
);

impl<'a> egui::Widget for ItemStackWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let head_item = self.0.head_item();
        let head_item_name = if let Some((_, name)) =
            head_item.and_then(|head_item| self.1.select_one::<(Item, Name)>(head_item))
        {
            name.0
        } else {
            ""
        };
        let label = if head_item.is_some() {
            format!(
                "{} ({})",
                head_item_name.chars().take(3).join(""),
                self.0.len()
            )
        } else {
            String::new()
        };
        let draggable_btn = egui::Button::new(label)
            .min_size(egui::Vec2 { x: 30., y: 30. })
            .sense(egui::Sense::drag());
        let draggable_btn = ui.add(draggable_btn);
        if draggable_btn.drag_started() {
            if let (Some(_), Some(egui::Pos2 { x, y })) =
                (head_item, draggable_btn.interact_pointer_pos())
            {
                self.3.item_drag.start(self.0.clone(), (x, y));
            }
        } else if draggable_btn.drag_released() {
            if let Some(egui::Pos2 { x, y }) = draggable_btn.interact_pointer_pos() {
                self.3.item_drag.stop((x, y));
            }
        }
        draggable_btn
    }
}

pub(super) struct EquipmentWidget<'a>(
    pub(super) &'a EntityRef,
    pub(super) &'a State,
    pub(super) &'a mut StateCommands,
    pub(super) &'a mut UiState,
);

impl<'a> egui::Widget for EquipmentWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        egui::Grid::new(format!("Equipment[{:?}]", self.0))
            .show(ui, |ui| {
                if let Some((equipment,)) = self.1.select_one::<(Equipment,)>(self.0) {
                    equipment.slots().for_each(|(slot, item_stack)| {
                        ui.label(format!("{:?}", slot));
                        ui.add(ItemStackWidget(item_stack, self.1, self.2, self.3));
                        ui.end_row();
                    })
                }
            })
            .response
    }
}

pub(super) struct StorageWidget<'a>(
    pub(super) &'a EntityRef,
    pub(super) &'a State,
    pub(super) &'a mut StateCommands,
    pub(super) &'a mut UiState,
);

impl<'a> egui::Widget for StorageWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        egui::Grid::new(format!("Storage[{:?}]", self.0))
            .show(ui, |ui| {
                if let Some((storage,)) = self.1.select_one::<(Storage,)>(self.0) {
                    storage.stacks().chunks(3).into_iter().for_each(|row| {
                        row.into_iter().for_each(|item_stack| {
                            ui.add(ItemStackWidget(item_stack, self.1, self.2, self.3));
                        });
                        ui.end_row();
                    })
                }
            })
            .response
    }
}

pub(super) struct NeedsWidget<'a>(
    pub(super) &'a EntityRef,
    pub(super) &'a State,
    pub(super) &'a mut StateCommands,
    pub(super) &'a mut UiState,
);

impl<'a> egui::Widget for NeedsWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        egui::Grid::new(format!("Needs[{:?}]", self.0))
            .show(ui, |ui| {
                if let Some((needs,)) = self.1.select_one::<(Needs,)>(self.0) {
                    needs.0.iter().for_each(|(need_type, need_status)| {
                        ui.label(format!("{:?}", need_type));
                        ui.label(format!("{}/{}", need_status.curr, need_status.max));
                        ui.end_row();
                    })
                }
            })
            .response
    }
}
