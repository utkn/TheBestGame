use std::{
    collections::{HashSet, VecDeque},
    marker::PhantomData,
};

use crate::{item::ItemInsights, physics::ColliderInsights, prelude::*};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    Multiply(f32),
    Add(f32),
    Set(f32),
}

/// Represents a component that can be effected by an [`Effector<Self>`].
/// Allows generating a corresponding [`Affected<Self>`] component on the same entity.
pub trait AffectibleComponent: Component {
    fn apply_effect(self, effect: Effect) -> Self;
}

impl AffectibleComponent for MaxSpeed {
    fn apply_effect(mut self, effect: Effect) -> Self {
        match effect {
            Effect::Multiply(val) => self.0 *= val,
            Effect::Add(val) => self.0 += val,
            Effect::Set(val) => self.0 = val,
        }
        self
    }
}

impl AffectibleComponent for Acceleration {
    fn apply_effect(mut self, effect: Effect) -> Self {
        match effect {
            Effect::Multiply(val) => self.0 *= val,
            Effect::Add(val) => self.0 += val,
            Effect::Set(val) => self.0 = val,
        }
        self
    }
}

/// A component representing another affected component.
#[derive(Clone, Debug)]
pub struct Affected<T: AffectibleComponent> {
    /// The initial state of the component.
    initial_state: Option<T>,
    /// The applied effects.
    effects: VecDeque<Effect>,
}

impl<T: AffectibleComponent> Default for Affected<T> {
    fn default() -> Self {
        Self {
            initial_state: None,
            effects: Default::default(),
        }
    }
}

impl<T: AffectibleComponent> Affected<T> {
    /// Computes the final state of the component using the saved effects.
    pub fn final_state(&self, init: T) -> T {
        self.effects
            .iter()
            .fold(init, |state, effect| state.apply_effect(*effect))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EffectorTarget {
    /// The effect will be applied to the storer of the `Effector`.
    Storer,
    /// The effect will be applied to the equipper of the `Effector`.
    Equipper,
    /// The effect will be applied to the entities colliding with this `Effector` during the collision.
    Collider,
}

/// A component representing an entity that can apply effects to other entities.
#[derive(Clone, Debug)]
pub struct Effector<T: AffectibleComponent> {
    effect: Effect,
    targets: HashSet<EffectorTarget>,
    pd: PhantomData<T>,
}

impl<T: AffectibleComponent> Effector<T> {
    pub fn new(targets: impl IntoIterator<Item = EffectorTarget>, effect: Effect) -> Self {
        Self {
            effect,
            targets: HashSet::from_iter(targets),
            pd: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ApplyEffectReq<T: AffectibleComponent>(EntityRef, Effect, PhantomData<T>);

#[derive(Clone, Copy, Debug)]
struct UnapplyEffectReq<T: Component>(EntityRef, Effect, PhantomData<T>);

#[derive(Clone, Copy, Debug)]
struct EffectAppliedEvt<T: Component>(EntityRef, Effect, PhantomData<T>);

#[derive(Clone, Copy, Debug)]
struct EffectUnappliedEvt<T: Component>(EntityRef, Effect, PhantomData<T>);

/// A system that handles effects that apply to type `T`.
#[derive(Clone, Copy, Debug)]
pub struct EffectSystem<T>(PhantomData<T>);

impl<T> Default for EffectSystem<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: AffectibleComponent, R: StateReader, W: StateWriter> System<R, W> for EffectSystem<T> {
    fn update(&mut self, ctx: &UpdateContext, state: &R, cmds: &mut W) {
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
                let insights = StateInsights::of(state);
                // Collect the application targets.
                let mut apply_targets = HashSet::<EntityRef>::new();
                let mut unapply_targets = HashSet::<EntityRef>::new();
                if effector.targets.contains(&EffectorTarget::Collider) {
                    apply_targets.extend(insights.new_collision_starters_of(&e));
                    unapply_targets.extend(insights.new_collision_enders_of(&e));
                }
                if effector.targets.contains(&EffectorTarget::Storer) {
                    apply_targets.extend(insights.new_storers_of(&e));
                    unapply_targets.extend(insights.new_unstorers_of(&e));
                }
                if effector.targets.contains(&EffectorTarget::Equipper) {
                    apply_targets.extend(insights.new_equippers_of(&e));
                    unapply_targets.extend(insights.new_unequippers_of(&e));
                }
                // Emit an application/unapplication request for the targets.
                unapply_targets.into_iter().for_each(|target| {
                    cmds.emit_event(UnapplyEffectReq::<T>(
                        target,
                        effector.effect,
                        Default::default(),
                    ));
                });
                apply_targets.into_iter().for_each(|target| {
                    cmds.emit_event(ApplyEffectReq::<T>(
                        target,
                        effector.effect,
                        Default::default(),
                    ));
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
