mod component;
mod component_tuple;
mod entity;
mod event;
mod generic_bag;
pub mod primitive_components;
mod state;
mod system;
mod template;

pub use entity::*;
pub use event::EventManager;
pub use state::*;
pub use system::*;
pub use template::EntityTemplate;

pub struct World {
    state: State,
    systems: Vec<Box<dyn System>>,
}

impl From<State> for World {
    fn from(state: State) -> Self {
        Self {
            state,
            systems: Default::default(),
        }
    }
}

impl World {
    pub fn get_state(&self) -> &State {
        &self.state
    }

    pub fn register_system<T: System>(&mut self, system: T) {
        self.systems.push(Box::new(system))
    }

    pub fn update_with(&mut self, initializer: impl FnOnce(&State, &mut StateCommands)) {
        let mut cmds = StateCommands::from(&self.state);
        initializer(&self.state, &mut cmds);
        self.state.apply_cmds(cmds)
    }

    pub fn update_with_systems(&mut self, update_ctx: UpdateContext) {
        let mut cmds = StateCommands::from(&self.state);
        for s in &mut self.systems {
            s.update(&update_ctx, &self.state, &mut cmds);
        }
        // Systems should consume all the events.
        self.state.clear_events();
        self.state.apply_cmds(cmds)
    }
}
