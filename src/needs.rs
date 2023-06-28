use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NeedType {
    Health,
    Energy,
    Hunger,
    Thirst,
    Sanity,
}

/// Contains the status of a need.
#[derive(Clone, Copy, Debug)]
pub struct NeedStatus {
    /// Previous value.
    pub prev: Option<f32>,
    /// Current value.
    pub curr: f32,
    /// The maximum value.
    pub max: f32,
}

impl NeedStatus {
    /// Creates a new `NeedStatus` that starts at the maximum value.
    pub fn with_max(max: f32) -> Self {
        assert!(max > 0.);
        Self {
            prev: None,
            curr: max,
            max,
        }
    }

    /// Creates a new `NeedStatus` that ends at the maximum value.
    pub fn with_zero(max: f32) -> Self {
        assert!(max > 0.);
        Self {
            prev: None,
            curr: 0.,
            max,
        }
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
        self.prev = Some(self.curr);
        self.curr += delta;
    }

    /// Returns the previous fraction.
    pub fn get_prev_frac(&self) -> Option<f32> {
        Some(self.prev? / self.max)
    }

    /// Returns the fraction, i.e., current value divided by the maximum value.
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

/// A system that monitors the state of the needs and outputs the appropriate events when they change.
#[derive(Clone, Debug)]
pub struct NeedStateSystem;

impl<R: StateReader, W: StateWriter> System<R, W> for NeedStateSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &R, cmds: &mut W) {
        // Handle need status changes...
        state.select::<(Needs,)>().for_each(|(e, (curr_needs,))| {
            curr_needs.0.iter().for_each(|(need_type, curr_status)| {
                // Get the new fraction.
                let new_frac = curr_status.get_fraction();
                // Get the old fraction.
                let old_frac = curr_status.get_prev_frac().unwrap_or(new_frac);
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

#[derive(Clone, Copy, Debug)]
pub enum NeedMutatorEffect {
    /// The given delta will be directly applied to the status.
    Delta(f32),
    /// The given rate will be multiplied by delta time before applying it to the status.
    Rate(f32),
}

#[derive(Clone, Debug)]
pub struct NeedMutator {
    need_type: NeedType,
    effect: NeedMutatorEffect,
}

impl NeedMutator {
    pub fn new(need_type: NeedType, effect: NeedMutatorEffect) -> Self {
        Self { need_type, effect }
    }
}

/// A system that changes the `Needs` of the entities through `NeedMutator`s applied to the entities.
#[derive(Clone, Copy, Debug)]
pub struct NeedMutatorSystem;

impl<R: StateReader, W: StateWriter> System<R, W> for NeedMutatorSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &R, cmds: &mut W) {
        state
            .select::<(NeedMutator,)>()
            .for_each(|(e, (mutator,))| {
                // Apply the effects.
                let need_type = mutator.need_type;
                let need_change = match mutator.effect {
                    NeedMutatorEffect::Delta(delta) => delta,
                    NeedMutatorEffect::Rate(rate) => rate * ctx.dt,
                };
                cmds.update_component(&e, move |needs: &mut Needs| {
                    needs
                        .get_mut(&need_type)
                        .map(|status| status.change(&need_change));
                });
            })
    }
}
