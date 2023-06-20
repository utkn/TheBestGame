use crate::prelude::*;

use super::{TryInteractReq, TryUninteractReq};

/// A component that converts interaction requests to its parent while acting as a target.
#[derive(Clone, Copy, Debug)]
pub struct InteractionDelegate(pub EntityRef);

#[derive(Clone, Copy, Debug)]
pub struct InteractionDelegateSystem;

impl System for InteractionDelegateSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Remove delegate components if the delegee is invalid.
        state
            .select::<(InteractionDelegate,)>()
            .for_each(|(delegee, (delegate,))| {
                if !state.is_valid(&delegate.0) {
                    cmds.remove_component::<InteractionDelegate>(&delegee);
                }
            });
        // Handle the try interact requests emitted for this delegate.
        state.read_events::<TryUninteractReq>().for_each(|evt| {
            if let Some((target_delegate,)) =
                state.select_one::<(InteractionDelegate,)>(&evt.target)
            {
                cmds.emit_event(TryUninteractReq::new(evt.actor, target_delegate.0));
            }
        });
        state.read_events::<TryInteractReq>().for_each(|evt| {
            if let Some((target_delegate,)) =
                state.select_one::<(InteractionDelegate,)>(&evt.target)
            {
                cmds.emit_event(TryInteractReq::new(evt.actor, target_delegate.0));
            }
        });
    }
}
