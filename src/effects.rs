use std::{
    collections::{HashSet, VecDeque},
    marker::PhantomData,
};

use crate::{core::*, entity_insights::EntityInsights};

pub type Effect<T> = fn(T) -> T;

#[derive(Clone, Debug)]
pub struct Affected<T: Component> {
    initial_state: Option<T>,
    effects: VecDeque<Effect<T>>,
}

impl<T: Component> Default for Affected<T> {
    fn default() -> Self {
        Self {
            initial_state: None,
            effects: Default::default(),
        }
    }
}

impl<T: Component> Affected<T> {
    pub fn final_state(&self, init: T) -> T {
        self.effects
            .iter()
            .fold(init, |state, effect| effect(state))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EffectorTarget {
    Storer,
    Equipper,
}

#[derive(Clone, Debug)]
pub struct Effector<T: Component> {
    effect: Effect<T>,
    targets: HashSet<EffectorTarget>,
}

impl<T: Component> Effector<T> {
    pub fn new(targets: impl IntoIterator<Item = EffectorTarget>, effect: Effect<T>) -> Self {
        Self {
            effect,
            targets: HashSet::from_iter(targets),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ApplyEffectReq<T: Component>(EntityRef, Effect<T>);

#[derive(Clone, Copy, Debug)]
struct UnapplyEffectReq<T: Component>(EntityRef, Effect<T>);

#[derive(Clone, Copy, Debug)]
struct EffectAppliedEvt<T: Component>(EntityRef, Effect<T>);

#[derive(Clone, Copy, Debug)]
struct EffectUnappliedEvt<T: Component>(EntityRef, Effect<T>);

#[derive(Clone, Copy, Debug)]
pub struct EffectSystem<T>(PhantomData<T>);

impl<T> Default for EffectSystem<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Component> System for EffectSystem<T> {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Apply the effects.
        state
            .select::<(Affected<T>, T)>()
            .for_each(|(e, (affected, target_component))| {
                // Either copy in the saved initial state, or read from the current state.
                let initial_state = affected
                    .initial_state
                    .clone()
                    .unwrap_or(target_component.clone());
                // Calculate the final state using the applied effects.
                let final_state = affected.final_state(initial_state.clone());
                // Update the saved initial state and the final state of the component.
                cmds.update_component(&e, move |affected: &mut Affected<T>| {
                    affected.initial_state = Some(initial_state);
                });
                cmds.set_component(&e, final_state);
            });
        // Emit effect application requests.
        state
            .select::<(Effector<T>,)>()
            .for_each(|(e, (effector,))| {
                // Collect insights about the effector.
                let effector_insights = EntityInsights::of(&e, state);
                // Collect the application targets.
                let mut apply_targets = HashSet::<EntityRef>::new();
                let mut unapply_targets = HashSet::<EntityRef>::new();
                if effector.targets.contains(&EffectorTarget::Storer) {
                    apply_targets.extend(effector_insights.storers);
                    unapply_targets.extend(effector_insights.unstorers);
                }
                if effector.targets.contains(&EffectorTarget::Equipper) {
                    apply_targets.extend(effector_insights.equippers);
                    unapply_targets.extend(effector_insights.unequippers);
                }
                // Emit an application/unapplication request for the targets.
                unapply_targets.into_iter().for_each(|target| {
                    cmds.emit_event(UnapplyEffectReq(target, effector.effect));
                });
                apply_targets.into_iter().for_each(|target| {
                    cmds.emit_event(ApplyEffectReq(target, effector.effect));
                });
            });
        // Handle effect application/unapplication requests. Yes, on the same system that emits them.
        state.read_events::<ApplyEffectReq<T>>().for_each(|evt| {
            if let Some(_) = state.select_one::<(Affected<T>, T)>(&evt.0) {
                let effect_to_apply = evt.1;
                cmds.update_component(&evt.0, move |affected: &mut Affected<T>| {
                    affected.effects.push_back(effect_to_apply);
                });
            }
        });
        state.read_events::<UnapplyEffectReq<T>>().for_each(|evt| {
            if let Some(_) = state.select_one::<(Affected<T>, T)>(&evt.0) {
                let effect_to_unapply = evt.1;
                cmds.update_component(&evt.0, move |affected: &mut Affected<T>| {
                    affected.effects = affected
                        .effects
                        .iter()
                        .cloned()
                        .filter(|effect| *effect != effect_to_unapply)
                        .collect();
                });
            }
        });
    }
}
