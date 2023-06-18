use std::collections::HashSet;

use crate::{
    core::*,
    entity_insights::EntityLocation,
    interaction::{InteractionEndedEvt, InteractionStartedEvt},
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
}

#[derive(Clone, Copy, Debug)]
pub struct ActivatedEvt(pub EntityRef);

#[derive(Clone, Copy, Debug)]
pub struct DeactivatedEvt(pub EntityRef);

#[derive(Clone, Copy, Debug)]
pub struct ActivationSystem;

impl System for ActivationSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state.read_events::<InteractionEndedEvt>().for_each(|evt| {
            if let Some((target_activ,)) = state.select_one::<(Activatable,)>(&evt.0.target) {
                if target_activ.curr_state {
                    cmds.emit_event(DeactivatedEvt(evt.0.target));
                    cmds.update_component(&evt.0.target, |activ: &mut Activatable| {
                        activ.curr_state = false;
                    })
                }
            }
        });
        state
            .read_events::<InteractionStartedEvt>()
            .for_each(|evt| {
                if let Some((target_activ,)) = state.select_one::<(Activatable,)>(&evt.0.target) {
                    let curr_loc = EntityLocation::of(&evt.0.target, state).into();
                    if !target_activ.curr_state && target_activ.locs.contains(&curr_loc) {
                        cmds.emit_event(ActivatedEvt(evt.0.target));
                        cmds.update_component(&evt.0.target, |activ: &mut Activatable| {
                            activ.curr_state = true;
                        })
                    }
                }
            });
    }
}
