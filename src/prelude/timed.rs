use std::marker::PhantomData;

use crate::prelude::*;

/// A wrapper compoonent that replaces itself with the inner component after a certain time.
#[derive(Clone, Debug)]
pub struct TimedAdd<T: Component> {
    remaining: f32,
    component: T,
}

impl<T: Component> TimedAdd<T> {
    /// Creates a `TimedAdd<T>` component that replaces itself with the inner component of type `T` after given time.
    pub fn new(time: f32, component_to_add: T) -> Self {
        Self {
            remaining: time,
            component: component_to_add,
        }
    }
}

/// A system that handles timed component additions of type `T`.
#[derive(Clone, Copy, Debug)]
pub struct TimedAddSystem<T: Component>(PhantomData<T>);

impl<T: Component> Default for TimedAddSystem<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Component, R: StateReader> System<R> for TimedAddSystem<T> {
    fn update(&mut self, ctx: &UpdateContext, state: &R, cmds: &mut StateCommands) {
        state
            .select::<(TimedAdd<T>,)>()
            .for_each(|(e, (timed_add,))| {
                // If the time has been reached...
                if timed_add.remaining <= 0. {
                    // ... replace the timed add component with its contents.
                    cmds.set_component(&e, timed_add.component.clone());
                    cmds.remove_component::<TimedAdd<T>>(&e);
                } else {
                    // Otherwise, decrease the lifetime.
                    let dt = ctx.dt;
                    cmds.update_component(&e, move |cooldown: &mut TimedAdd<T>| {
                        cooldown.remaining -= dt;
                    });
                }
            });
    }
}

/// A wrapper compoonent that removes itself and the component `T` after a certain time.
#[derive(Clone, Debug)]
pub struct TimedRemove<T: Component> {
    remaining: f32,
    pd: PhantomData<T>,
}

impl<T: Component> TimedRemove<T> {
    /// Creates a component that removes itself and the given component type after given time.
    pub fn new(time: f32) -> Self {
        Self {
            remaining: time,
            pd: Default::default(),
        }
    }
}

/// A system that handles timed component removals of type `T`.
#[derive(Clone, Copy, Debug)]
pub struct TimedRemoveSystem<T: Component>(PhantomData<T>);

impl<T: Component> Default for TimedRemoveSystem<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Component, R: StateReader> System<R> for TimedRemoveSystem<T> {
    fn update(&mut self, ctx: &UpdateContext, state: &R, cmds: &mut StateCommands) {
        state
            .select::<(TimedRemove<T>,)>()
            .for_each(|(e, (timed_remove,))| {
                // If the time has been reached...
                if timed_remove.remaining <= 0. {
                    // ... remove the components.
                    cmds.remove_component::<T>(&e);
                    cmds.remove_component::<TimedRemove<T>>(&e);
                } else {
                    // Otherwise, decrease the lifetime.
                    let dt = ctx.dt;
                    cmds.update_component(&e, move |cooldown: &mut TimedRemove<T>| {
                        cooldown.remaining -= dt;
                    });
                }
            });
    }
}

/// A wrapper compoonent that emits an event and removes itself after given time.
#[derive(Clone, Debug)]
pub struct TimedEmit<T: Event> {
    remaining: f32,
    event: T,
}

impl<T: Component> TimedEmit<T> {
    pub fn new(time: f32, event: T) -> Self {
        Self {
            remaining: time,
            event,
        }
    }
}

/// A system that handles timed event emits of type `T`.
#[derive(Clone, Copy, Debug)]
pub struct TimedEmitSystem<T: Component>(PhantomData<T>);

impl<T: Component> Default for TimedEmitSystem<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Component, R: StateReader> System<R> for TimedEmitSystem<T> {
    fn update(&mut self, ctx: &UpdateContext, state: &R, cmds: &mut StateCommands) {
        state
            .select::<(TimedEmit<T>,)>()
            .for_each(|(e, (timed_emit,))| {
                // If the time has been reached...
                if timed_emit.remaining <= 0. {
                    cmds.remove_component::<TimedEmit<T>>(&e);
                    cmds.emit_event(timed_emit.event.clone());
                } else {
                    // Otherwise, decrease the lifetime.
                    let dt = ctx.dt;
                    cmds.update_component(&e, move |cooldown: &mut TimedEmit<T>| {
                        cooldown.remaining -= dt;
                    });
                }
            });
    }
}
