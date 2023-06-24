mod basic_components;
mod basic_systems;
mod component;
mod component_tuple;
mod entity;
mod event;
mod generic_bag;
mod interaction;
mod state;
mod state_insights;
mod system;
mod tags;
mod timed;

pub use basic_components::*;
pub use basic_systems::*;
pub use component::Component;
pub use entity::*;
pub use event::Event;
pub use interaction::*;
pub use state::{State, StateCommands};
pub use state_insights::*;
pub use system::*;
pub use tags::*;
pub use timed::*;

/// Represents the game world.
#[derive(Debug)]
pub struct World {
    /// The state of the world.
    state: State,
    /// The list of registered systems that read & update the state.
    systems: Vec<Box<dyn System>>,
}

impl From<State> for World {
    /// Creates a game world with no systems from the given game state.
    fn from(state: State) -> Self {
        Self {
            state,
            systems: Default::default(),
        }
    }
}

impl World {
    /// Returns the current state of the world.
    pub fn get_state(&self) -> &State {
        &self.state
    }

    /// Registers a new system to this world.
    pub fn register_system<T: System>(&mut self, system: T) {
        self.systems.push(Box::new(system))
    }

    /// Updates the state of the world with the given closure eagerly.
    pub fn update_with(&mut self, initializer: impl FnOnce(&State, &mut StateCommands)) {
        let mut cmds = StateCommands::from(&self.state);
        initializer(&self.state, &mut cmds);
        self.state.apply_cmds(cmds)
    }

    /// Updates the state of the world with the registered systems.
    pub fn update_with_systems(&mut self, update_ctx: UpdateContext) {
        let mut cmds = StateCommands::from(&self.state);
        // Take in the removals.
        self.state.transfer_removals(&mut cmds);
        for s in &mut self.systems {
            s.update(&update_ctx, &self.state, &mut cmds);
        }
        // Systems should consume all the events.
        self.state.clear_events();
        self.state.reset_removal_requests();
        self.state.apply_cmds(cmds)
    }
}
