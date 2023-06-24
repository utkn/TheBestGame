use crate::prelude::{EntityRef, StateInsights};

use super::Equipment;

pub trait EquipmentInsights {
    fn is_equipment(&self, e: &EntityRef) -> bool;
}

impl<'a> EquipmentInsights for StateInsights<'a> {
    fn is_equipment(&self, e: &EntityRef) -> bool {
        self.0.select_one::<(Equipment,)>(e).is_some()
    }
}
