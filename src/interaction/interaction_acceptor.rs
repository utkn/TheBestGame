use itertools::Itertools;

use crate::prelude::*;

use super::ProposeInteractionEvt;

pub struct InteractionAcceptorSystem;

impl System for InteractionAcceptorSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        let consensus_instances = state
            .read_events::<ProposeInteractionEvt>()
            .into_group_map_by(|evt| evt.consensus_id);
        consensus_instances.into_iter().for_each(|(_, proposals)| {
            let picked_proposal = proposals
                .into_iter()
                .max_by_key(|proposal| proposal.priority);
            if let Some(picked_proposal) = picked_proposal {
                cmds.emit_event(InteractionAcceptedEvt::from(picked_proposal));
            }
        });
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct InteractionAcceptedEvt {
    pub(super) consensus_id: usize,
    /// The unique type id of the proposer. Used to distinguish between different proposals for the same request.
    pub(super) proposer_tid: std::any::TypeId,
    pub(super) actor: EntityRef,
    pub(super) target: EntityRef,
}

impl From<&ProposeInteractionEvt> for InteractionAcceptedEvt {
    /// Constructs from the given proposal.
    fn from(proposal: &ProposeInteractionEvt) -> Self {
        InteractionAcceptedEvt {
            consensus_id: proposal.consensus_id,
            proposer_tid: proposal.proposer_id,
            actor: proposal.actor,
            target: proposal.target,
        }
    }
}
