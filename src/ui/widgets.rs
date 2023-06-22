use itertools::Itertools;
use notan::egui;

use crate::{
    item::{Equipment, EquipmentSlot, Item, Storage},
    needs::Needs,
    prelude::*,
};

use super::UiState;

pub(super) struct ItemWidget<'a>(
    pub(super) &'a EntityRef,
    pub(super) &'a State,
    pub(super) &'a mut StateCommands,
    pub(super) &'a mut UiState,
);

impl<'a> egui::Widget for ItemWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let name = if let Some((_, name)) = self.1.select_one::<(Item, Name)>(self.0) {
            name.0
        } else {
            "unknown"
        };
        let draggable_btn = egui::Button::new(name.chars().take(3).join(""))
            .min_size(egui::Vec2 { x: 30., y: 30. })
            .sense(egui::Sense::drag());
        let draggable_btn = ui.add(draggable_btn);
        if draggable_btn.drag_started() {
            if let Some(egui::Pos2 { x, y }) = draggable_btn.interact_pointer_pos() {
                self.3.item_drag.start(*self.0, (x, y));
            }
        } else if draggable_btn.drag_released() {
            if let Some(egui::Pos2 { x, y }) = draggable_btn.interact_pointer_pos() {
                self.3.item_drag.stop((x, y));
            }
        }
        draggable_btn
    }
}

pub(super) struct EquipmentSlotWidget<'a>(
    pub(super) EquipmentSlot,
    pub(super) Option<&'a EntityRef>,
    pub(super) &'a State,
    pub(super) &'a mut StateCommands,
    pub(super) &'a mut UiState,
);

impl<'a> egui::Widget for EquipmentSlotWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        egui::Grid::new(format!("EquipmentSlot[{:?}]", self.0))
            .show(ui, |ui| {
                if let Some(item) = self.1 {
                    ui.add(ItemWidget(item, self.2, self.3, self.4))
                } else {
                    ui.add(egui::Button::new("").min_size(egui::Vec2 { x: 30., y: 30. }))
                }
            })
            .response
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
        let equipment_grid = [
            [None, Some(EquipmentSlot::Head), None],
            [
                Some(EquipmentSlot::LeftHand),
                Some(EquipmentSlot::Torso),
                Some(EquipmentSlot::RightHand),
            ],
            [
                Some(EquipmentSlot::Legs),
                Some(EquipmentSlot::Backpack),
                None,
            ],
            [None, Some(EquipmentSlot::Feet), None],
        ];
        egui::Grid::new(format!("Equipment[{:?}]", self.0))
            .show(ui, |ui| {
                if let Some((equipment,)) = self.1.select_one::<(Equipment,)>(self.0) {
                    for slots_row in equipment_grid {
                        for opt_slot in slots_row {
                            if let Some(slot) = opt_slot {
                                let item_in_slot = equipment
                                    .get_item_stack(&slot)
                                    .and_then(|item_stack| item_stack.head_item());
                                ui.add(EquipmentSlotWidget(
                                    slot,
                                    item_in_slot,
                                    self.1,
                                    self.2,
                                    self.3,
                                ));
                            } else {
                                ui.label("");
                            }
                        }
                        ui.end_row();
                    }
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
                    storage
                        .stacks()
                        .flat_map(|stack| stack.head_item())
                        .chunks(3)
                        .into_iter()
                        .for_each(|row| {
                            row.into_iter().for_each(|item| {
                                ui.add(ItemWidget(item, self.1, self.2, self.3));
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
