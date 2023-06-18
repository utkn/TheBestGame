use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use crate::{
    core::*,
    item::EntityLocation,
    physics::{CollisionEvt, CollisionStartEvt, CollisionState},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NeedType {
    Health,
    Hunger,
    Thirst,
    Sanity,
}

#[derive(Clone, Copy, Debug)]
pub struct NeedStatus {
    pub curr: f32,
    pub max: f32,
}

impl NeedStatus {
    /// Creates a new `NeedStatus` that starts at the maximum value.
    pub fn with_max(max: f32) -> Self {
        assert!(max > 0.);
        Self { curr: max, max }
    }

    /// Creates a new `NeedStatus` that ends at the maximum value.
    pub fn with_zero(max: f32) -> Self {
        assert!(max > 0.);
        Self { curr: 0., max }
    }

    /// Sets the current status to the maximum value.
    pub fn maximize(&mut self) {
        self.curr = self.max;
    }

    /// Sets the current status to zero.
    pub fn zero(&mut self) {
        self.curr = 0.;
    }

    /// Applies the given change to the status.
    pub fn change(&mut self, delta: &f32) {
        self.curr += delta;
    }

    pub fn get_fraction(&self) -> f32 {
        if self.max == 0. {
            return 0.;
        }
        (self.curr as f32) / (self.max as f32)
    }
}

/// Represents a change in the status of a need.
#[derive(Clone, Copy, Debug)]
pub enum NeedChange {
    Decreased(f32, f32),
    Increased(f32, f32),
    ExceededMaximum(f32, f32),
    DescendedZero(f32, f32),
}

#[derive(Clone, Debug)]
pub struct Needs(pub Vec<(NeedType, NeedStatus)>);

impl Needs {
    pub fn get(&self, t: &NeedType) -> Option<&NeedStatus> {
        self.0
            .iter()
            .find(|(need_type, _)| need_type == t)
            .map(|(_, status)| status)
    }

    pub fn get_mut(&mut self, t: &NeedType) -> Option<&mut NeedStatus> {
        self.0
            .iter_mut()
            .find(|(need_type, _)| need_type == t)
            .map(|(_, status)| status)
    }
}

impl Needs {
    pub fn new(needs_iter: impl IntoIterator<Item = (NeedType, NeedStatus)>) -> Self {
        Self(Vec::from_iter(needs_iter))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NeedChangeEvt(EntityRef, NeedType, NeedChange);

#[derive(Clone, Debug, Default)]
pub struct NeedsSystem {
    old_state: HashMap<EntityRef, Needs>,
}

impl System for NeedsSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Remove the invalidated entities from the entities for which we maintain needs.
        let invalids = self
            .old_state
            .keys()
            .filter(|e| state.select_one::<(Needs,)>(e).is_none())
            .cloned()
            .collect_vec();
        invalids.into_iter().for_each(|invalid| {
            self.old_state.remove(&invalid);
        });
        // Handle need status changes...
        state.select::<(Needs,)>().for_each(|(e, (curr_needs,))| {
            let old_needs = self.old_state.entry(e).or_insert(curr_needs.clone());
            curr_needs.0.iter().for_each(|(need_type, curr_status)| {
                // Get the old fraction.
                let old_frac = old_needs.get(need_type).map(|s| s.get_fraction());
                let old_frac = old_frac.unwrap_or_default();
                // Get the new fraction.
                let new_frac = curr_status.get_fraction();
                // Emit the appropriate increased/decreased event.
                let need_change = if new_frac > old_frac {
                    NeedChange::Increased(old_frac, new_frac)
                } else if new_frac < old_frac {
                    NeedChange::Decreased(old_frac, new_frac)
                } else {
                    return;
                };
                let need_type = *need_type;
                cmds.emit_event(NeedChangeEvt(e, need_type, need_change));
                // If the need has exceeded the maximum, set the need to maximum and emit the appropriate event.
                if new_frac > 1. {
                    let exceeded_change = NeedChange::ExceededMaximum(old_frac, new_frac);
                    cmds.emit_event(NeedChangeEvt(e, need_type, exceeded_change));
                    cmds.update_component(&e, move |needs: &mut Needs| {
                        needs.get_mut(&need_type).map(|need| need.maximize());
                    });
                }
                // If the need has descended zero, set the need to zero and emit the appropriate event.
                if new_frac < 0. {
                    let descended_change = NeedChange::DescendedZero(old_frac, new_frac);
                    cmds.emit_event(NeedChangeEvt(e, need_type, descended_change));
                    cmds.update_component(&e, move |needs: &mut Needs| {
                        needs.get_mut(&need_type).map(|need| need.zero());
                    });
                }
            });
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum NeedEffectorCond {
    OnCollision,
    OnCollisionStart,
    InEquipment,
    InStorage,
}

#[derive(Clone, Debug)]
pub struct NeedEffector {
    conds: HashSet<NeedEffectorCond>,
    need_type: NeedType,
    effect_rate: f32,
}

impl NeedEffector {
    pub fn new(
        conditions: impl IntoIterator<Item = NeedEffectorCond>,
        need_type: NeedType,
        effect_rate: f32,
    ) -> Self {
        Self {
            conds: HashSet::from_iter(conditions),
            need_type,
            effect_rate,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NeedEffectorSystem;

impl System for NeedEffectorSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(NeedEffector,)>()
            .for_each(|(e, (effector,))| {
                let effector_location = EntityLocation::of(&e, state);
                let effector_colliders: HashSet<_> = state
                    .read_events::<CollisionEvt>()
                    .filter(|evt| evt.e1 == e)
                    .map(|evt| evt.e2)
                    .collect();
                let effector_collision_starters: HashSet<_> = state
                    .read_events::<CollisionStartEvt>()
                    .filter(|evt| evt.e1 == e)
                    .map(|evt| evt.e2)
                    .collect();
                // Collect the targets to apply the effect to.
                let mut targets = HashSet::<EntityRef>::new();
                if effector.conds.contains(&NeedEffectorCond::OnCollisionStart) {
                    targets.extend(effector_collision_starters.into_iter());
                }
                if effector.conds.contains(&NeedEffectorCond::OnCollision) {
                    targets.extend(effector_colliders.into_iter());
                }
                if effector.conds.contains(&NeedEffectorCond::InEquipment) {
                    if let EntityLocation::Equipment(equipping_entity) = effector_location {
                        targets.insert(equipping_entity);
                    }
                }
                if effector.conds.contains(&NeedEffectorCond::InStorage) {
                    if let EntityLocation::Storage(storing_entity) = effector_location {
                        targets.insert(storing_entity);
                    }
                }
                // Apply the effects.
                let need_type = effector.need_type;
                let need_change = effector.effect_rate * ctx.dt;
                targets.into_iter().for_each(|target| {
                    cmds.update_component(&target, move |target_needs: &mut Needs| {
                        target_needs
                            .get_mut(&need_type)
                            .map(|status| status.change(&need_change));
                    });
                });
            })
    }
}
