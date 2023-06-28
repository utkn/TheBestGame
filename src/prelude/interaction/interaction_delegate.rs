use crate::prelude::*;

use super::{TryInteractReq, TryUninteractReq};

/// A component that converts untargeted interact/uninteract requests to its parent while acting as a target.
#[derive(Clone, Copy, Debug)]
pub struct UntargetedInteractionDelegate(pub EntityRef);

#[derive(Clone, Copy, Debug)]
pub struct UntargetedInteractionDelegateSystem;

impl<R: StateReader, W: StateWriter> System<R, W> for UntargetedInteractionDelegateSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &R, cmds: &mut W) {
        // Remove delegate components if the delegee is invalid.
        state
            .select::<(UntargetedInteractionDelegate,)>()
            .for_each(|(delegee, (delegate,))| {
                if !state.is_valid(&delegate.0) {
                    cmds.remove_component::<UntargetedInteractionDelegate>(&delegee);
                }
            });
        // Handle the try interact requests emitted for this delegate.
        state.read_events::<TryUninteractReq>().for_each(|evt| {
            if let Some((target_delegate,)) =
                state.select_one::<(UntargetedInteractionDelegate,)>(&evt.target)
            {
                cmds.emit_event(TryUninteractReq::new(evt.actor, target_delegate.0));
            }
        });
        state.read_events::<TryInteractReq>().for_each(|evt| {
            if let Some((target_delegate,)) =
                state.select_one::<(UntargetedInteractionDelegate,)>(&evt.target)
            {
                cmds.emit_event(TryInteractReq::new(evt.actor, target_delegate.0));
            }
        });
    }
}
