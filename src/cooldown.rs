use std::marker::PhantomData;

use crate::core::*;

/// A wrapper compoonent that replaces itself with the inner component after a certain time.
#[derive(Clone, Debug)]
pub struct Cooldown<T: Component> {
    remaining: f32,
    component: T,
}

impl<T: Component> Cooldown<T> {
    /// Creates a cooldown component that replaces itself with the inner component after given time.
    pub fn new(time: f32, component_to_add: T) -> Self {
        Self {
            remaining: time,
            component: component_to_add,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CooldownSystem<T: Component>(PhantomData<T>);

impl<T: Component> Default for CooldownSystem<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Component> System for CooldownSystem<T> {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(Cooldown<T>,)>()
            .for_each(|(e, (cooldown,))| {
                // If the cooldown time has been reached...
                if cooldown.remaining <= 0. {
                    // ... replace the cooldown component with its contents.
                    cmds.set_component(&e, cooldown.component.clone());
                    cmds.remove_component::<Cooldown<T>>(&e);
                } else {
                    // Otherwise, decrease the lifetime.
                    let dt = ctx.dt;
                    cmds.update_component(&e, move |cooldown: &mut Cooldown<T>| {
                        cooldown.remaining -= dt;
                    });
                }
            });
    }
}
