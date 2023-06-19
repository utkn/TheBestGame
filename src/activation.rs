use std::{collections::HashSet, marker::PhantomData};

use crate::{
    core::*,
    entity_insights::EntityLocation,
    interaction::{Interactable, InteractionType},
};

/// Represents a location from where an entity can be activated.
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

/// An activatable entity. The activation is performed by interacting with the entity.
#[derive(Clone, Debug)]
pub struct Activatable<T: ActivatableComponent> {
    pub curr_state: bool,
    pd: PhantomData<T>,
}

impl<T: ActivatableComponent> Default for Activatable<T> {
    fn default() -> Self {
        Self {
            curr_state: false,
            pd: Default::default(),
        }
    }
}

impl<T: ActivatableComponent> Activatable<T> {
    /// Returns the inactivated version of this `Activatable` component.
    pub fn deactivated(mut self) -> Self {
        self.curr_state = false;
        self
    }
}

/// Emitted when an entity is activated.
#[derive(Clone, Copy, Debug)]
pub struct ActivatedEvt<T: Component> {
    pub activatable: EntityRef,
    pd: PhantomData<T>,
}

impl<T: Component> ActivatedEvt<T> {
    pub fn new(activatable: EntityRef) -> Self {
        Self {
            activatable,
            pd: Default::default(),
        }
    }
}

/// Emitted when an entity is deactivated.
#[derive(Clone, Copy, Debug)]
pub struct DeactivatedEvt<T: Component> {
    pub activatable: EntityRef,
    pd: PhantomData<T>,
}

impl<T: Component> DeactivatedEvt<T> {
    pub fn new(activatable: EntityRef) -> Self {
        Self {
            activatable,
            pd: Default::default(),
        }
    }
}

/// Represents a component that can be activated or deactivated.
/// Implementing this for `T` allows constructing [`Activatable<T>`] and [`ActivationInteraction<T>`].
pub trait ActivatableComponent: Component {
    fn can_activate(
        actor: &EntityRef,
        target: &EntityRef,
        target_component: &Self,
        state: &State,
    ) -> bool;
    fn activation_priority() -> usize;
}

/// An interaction type that handles activation/deactivation of [`Activatable<C>`] components.
#[derive(Debug, Clone)]
pub struct ActivationInteraction<C: ActivatableComponent>(PhantomData<C>);

impl<C: ActivatableComponent> InteractionType for ActivationInteraction<C> {
    fn valid_actors(target: &EntityRef, state: &State) -> Option<HashSet<EntityRef>> {
        None
    }

    fn can_start(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        match state.select_one::<(C, Activatable<C>)>(target) {
            Some((c, activatable)) => {
                !activatable.curr_state && C::can_activate(actor, target, c, state)
            }
            None => false,
        }
    }

    fn should_end(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        match state.select_one::<(C, Activatable<C>)>(target) {
            Some((c, activatable)) => {
                activatable.curr_state && !C::can_activate(actor, target, c, state)
            }
            None => true,
        }
    }

    fn on_start(_actor: &EntityRef, target: &EntityRef, _state: &State, cmds: &mut StateCommands) {
        cmds.emit_event(ActivatedEvt::<C>::new(*target));
        cmds.update_component(target, |activatable: &mut Activatable<C>| {
            activatable.curr_state = true;
        })
    }

    fn on_end(_actor: &EntityRef, target: &EntityRef, _state: &State, cmds: &mut StateCommands) {
        cmds.emit_event(DeactivatedEvt::<C>::new(*target));
        cmds.update_component(&target, |activatable: &mut Activatable<C>| {
            activatable.curr_state = false;
        })
    }

    fn priority() -> usize {
        C::activation_priority()
    }
}
