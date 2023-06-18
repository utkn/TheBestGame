use std::collections::HashSet;

use crate::{
    core::*,
    entity_insights::{EntityInsights, EntityLocation},
    interaction::Interactable,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActivationLoc {
    Ground,
    Equipment,
    Storage,
}

impl From<EntityLocation> for ActivationLoc {
    fn from(item_loc: EntityLocation) -> Self {
        match item_loc {
            EntityLocation::Ground => Self::Ground,
            EntityLocation::Equipment(_) => Self::Equipment,
            EntityLocation::Storage(_) => Self::Storage,
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Activatable {
    pub locs: HashSet<ActivationLoc>,
    pub curr_state: bool,
}

impl Activatable {
    /// Creates an activatable that can be activated at the given locations.
    pub fn at_locations(locs: impl IntoIterator<Item = ActivationLoc>) -> Self {
        Self {
            locs: HashSet::from_iter(locs),
            curr_state: false,
        }
    }

    /// Returns the inactivated version of this `Activatable` component.
    pub fn deactivated(mut self) -> Self {
        self.curr_state = false;
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ActivatedEvt(pub EntityRef);

#[derive(Clone, Copy, Debug)]
pub struct DeactivatedEvt(pub EntityRef);

#[derive(Clone, Copy, Debug)]
pub struct ActivationSystem;

impl System for ActivationSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state.select::<(Activatable, Interactable)>().for_each(
            |(e, (activatable, interactable))| {
                let activatable_loc = EntityInsights::of(&e, state).location.into();
                // Is the entity being interacted with ?
                let is_being_interacted = interactable.actors.len() > 0;
                // Is the entity in a location that may lead to an activation ?
                let is_in_activatable_location = activatable.locs.contains(&activatable_loc);
                // Determine if we should activate or not.
                let should_activate = is_being_interacted && is_in_activatable_location;
                // Toggle the activation status and emit the appropriate event.
                if should_activate && !activatable.curr_state {
                    cmds.emit_event(ActivatedEvt(e));
                    cmds.update_component(&e, |activ: &mut Activatable| {
                        activ.curr_state = true;
                    })
                } else if !should_activate && activatable.curr_state {
                    cmds.emit_event(DeactivatedEvt(e));
                    cmds.update_component(&e, |activ: &mut Activatable| {
                        activ.curr_state = false;
                    })
                }
            },
        );
    }
}
