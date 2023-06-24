use crate::prelude::{EntityRef, StateInsights};

use super::Equipment;

pub trait EquipmentInsights {
    /// Returns true iff the given entity has an equipment.
    fn has_equipment(&self, e: &EntityRef) -> bool;
}

impl<'a> EquipmentInsights for StateInsights<'a> {
    fn has_equipment(&self, e: &EntityRef) -> bool {
        self.0.select_one::<(Equipment,)>(e).is_some()
    }
}
