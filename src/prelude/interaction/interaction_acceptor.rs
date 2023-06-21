use itertools::Itertools;

use crate::prelude::*;

use super::ProposeInteractEvt;

#[derive(Clone, Copy, Debug)]
pub enum ConsensusStrategy {
    MaxPriority,
    AlwaysFail,
    MinPriority,
}

/// Represents a received proposal.
#[derive(Clone, Copy, Debug)]
struct Proposal {
    consensus_id: usize,
    proposer_id: std::any::TypeId,
    priority: usize,
    actor: EntityRef,
    target: EntityRef,
}

impl From<&ProposeInteractEvt> for Proposal {
    fn from(value: &ProposeInteractEvt) -> Self {
        Self {
            consensus_id: value.consensus_id,
            proposer_id: value.proposer_id,
            priority: value.priority,
            actor: value.actor,
            target: value.target,
        }
    }
}

impl From<&ProposeUninteractEvt> for Proposal {
    fn from(value: &ProposeUninteractEvt) -> Self {
        Self {
            consensus_id: value.consensus_id,
            proposer_id: value.proposer_id,
            priority: value.priority,
            actor: value.actor,
            target: value.target,
        }
    }
}

impl ConsensusStrategy {
    fn choose_proposal<'a>(&self, proposals: &[Proposal]) -> Option<Proposal> {
        match self {
            Self::MaxPriority => {
                let picked_proposal = proposals
                    .iter()
                    .map(|p| *p)
                    .max_by_key(|proposal| proposal.priority);
                picked_proposal
            }
            Self::MinPriority => {
                let picked_proposal = proposals
                    .iter()
                    .map(|p| *p)
                    .min_by_key(|proposal| proposal.priority);
                picked_proposal
            }
            Self::AlwaysFail => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InteractionAcceptorSystem(pub ConsensusStrategy, pub ConsensusStrategy);

impl System for InteractionAcceptorSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Interact proposals.
        let consensus_instances = state
            .read_events::<ProposeInteractEvt>()
            .into_group_map_by(|evt| evt.consensus_id);
        consensus_instances.into_iter().for_each(|(_, proposals)| {
            // println!("consensus {:?}: {:?}", consensus_id, proposals);
            let proposals = proposals.into_iter().map_into().collect_vec();
            let picked_proposal = self.0.choose_proposal(&proposals);
            if let Some(picked_proposal) = picked_proposal {
                cmds.emit_event(InteractAcceptedEvt::from(picked_proposal));
            } else {
                // cmds.emit_event(ConsensusFailedEvt {
                //     consensus_id,
                //     proposals: proposals.into_iter().cloned().collect(),
                // })
            }
        });
        // Uninteract proposals.
        let consensus_instances = state
            .read_events::<ProposeUninteractEvt>()
            .into_group_map_by(|evt| evt.consensus_id);
        consensus_instances.into_iter().for_each(|(_, proposals)| {
            // println!("consensus {:?}: {:?}", consensus_id, proposals);
            let proposals = proposals.into_iter().map_into().collect_vec();
            let picked_proposal = self.1.choose_proposal(&proposals);
            if let Some(picked_proposal) = picked_proposal {
                cmds.emit_event(UninteractAcceptedEvt::from(picked_proposal));
            }
        });
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct InteractAcceptedEvt {
    pub(super) consensus_id: usize,
    /// The unique type id of the proposer. Used to distinguish between different proposals for the same request.
    pub(super) proposer_tid: std::any::TypeId,
    pub(super) actor: EntityRef,
    pub(super) target: EntityRef,
}

impl From<Proposal> for InteractAcceptedEvt {
    /// Constructs from the given proposal.
    fn from(proposal: Proposal) -> Self {
        Self {
            consensus_id: proposal.consensus_id,
            proposer_tid: proposal.proposer_id,
            actor: proposal.actor,
            target: proposal.target,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct UninteractAcceptedEvt {
    pub(super) consensus_id: usize,
    /// The unique type id of the proposer. Used to distinguish between different proposals for the same request.
    pub(super) proposer_tid: std::any::TypeId,
    pub(super) actor: EntityRef,
    pub(super) target: EntityRef,
}

impl From<Proposal> for UninteractAcceptedEvt {
    /// Constructs from the given proposal.
    fn from(proposal: Proposal) -> Self {
        Self {
            consensus_id: proposal.consensus_id,
            proposer_tid: proposal.proposer_id,
            actor: proposal.actor,
            target: proposal.target,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConsensusFailedEvt {
    pub consensus_id: usize,
    pub proposals: Vec<ProposeInteractEvt>,
}
