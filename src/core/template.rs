use super::{component_tuple::ComponentTuple, EntityRef, StateCommands};

pub trait EntityTemplate<'a>: ComponentTuple<'a> {
    fn create(self, cmds: &mut StateCommands) -> EntityRef {
        let e = cmds.create_entity();
        cmds.set_components(&e, self);
        e
    }
}

impl<'a, T> EntityTemplate<'a> for T where T: ComponentTuple<'a> {}
