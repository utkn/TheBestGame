use std::collections::HashSet;

use itertools::Itertools;

use crate::prelude::*;

use super::{ColliderInsights, EffectiveHitbox, Hitbox};

/// Entities tagged with this component will initiate interactions with the entities that collide and are visible from the position of this entity.
#[derive(Clone, Copy, Debug)]
pub struct VisionField(pub f32);

impl Interaction for VisionField {
    fn priority() -> usize {
        0
    }

    /// Returns true. Can only be started explicitly by targeted requests.
    fn can_start_targeted(_actor: &EntityRef, _target: &EntityRef, _state: &State) -> bool {
        true
    }

    /// Returns false. Can only be started explicitly by targeted requests.
    fn can_start_untargeted(_actor: &EntityRef, _target: &EntityRef, _state: &State) -> bool {
        false
    }

    /// Can only be ended explicitly by targeted requests.
    fn can_end_untargeted(_actor: &EntityRef, _target: &EntityRef, _state: &State) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VisionSystem;

impl System for VisionSystem {
    fn update(&mut self, _ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state.select::<(VisionField,)>().for_each(|(e, _)| {
            StateInsights::of(state)
                .new_collision_enders_of(&e)
                .into_iter()
                .for_each(|target| {
                    cmds.emit_event(UninteractReq::<VisionField>::new(e, *target));
                });
        });
        state
            .select::<(VisionField, Transform)>()
            .for_each(|(vf_entity, (_, vf_trans))| {
                let insights = StateInsights::of(state);
                let ref_trans = insights
                    .anchor_parent_of(&vf_entity)
                    .and_then(|anchor| insights.transform_of(&anchor))
                    .unwrap_or(vf_trans);
                let vf_anchor_parent = StateInsights::of(state).anchor_parent_of(&vf_entity);
                let colliding_entities: HashSet<_> = StateInsights::of(state)
                    .contacts_of(&vf_entity)
                    .map(|contacts| {
                        contacts
                            .iter()
                            // Do not consider the anchor parent in the vision.
                            .filter(|colliding_e| {
                                let is_anchor_parent = vf_anchor_parent
                                    .map(|anchor_parent| anchor_parent == *colliding_e)
                                    .unwrap_or(false);
                                !is_anchor_parent
                            })
                            .filter(|colliding_e| {
                                state
                                    .select_one::<(InteractTarget<VisionField>,)>(colliding_e)
                                    .is_some()
                            })
                            .cloned()
                            .collect()
                    })
                    .unwrap_or_default();
                let colliding_hitboxes = colliding_entities
                    .iter()
                    .filter_map(|colliding_e| {
                        state
                            .select_one::<(Hitbox, Transform)>(colliding_e)
                            .map(|(_, trans)| {
                                (
                                    *colliding_e,
                                    trans,
                                    EffectiveHitbox::new(colliding_e, state).unwrap(),
                                )
                            })
                    })
                    .collect_vec();
                // Draw lines from the vision field reference position to the entities it is colliding with.
                let lines_to_colliders = colliding_hitboxes
                    .iter()
                    .map(|(colliding_e, trans, _)| {
                        (*colliding_e, notan::math::vec2(trans.x, trans.y))
                    })
                    .collect_vec();
                // Get the unobstructed entities.
                let unobstructed_entities: HashSet<_> = lines_to_colliders
                    .into_iter()
                    .filter(|(colliding_e, target_pos)| {
                        // Make sure that the `target` entity is not obstructed by any other entity.
                        colliding_hitboxes
                            .iter()
                            // Do not consider itself as a blocker.
                            .filter(|(e, _, _)| e != colliding_e)
                            // Only concrete hitboxes can block views.
                            .filter(|(_, _, ehb)| ehb.hb.0.is_concrete())
                            .all(|(_, _, ehb)| {
                                !sepax2d::line::intersects_segment(
                                    ehb.shape.shape_ref(),
                                    (ref_trans.x, ref_trans.y),
                                    (target_pos.x, target_pos.y),
                                )
                            })
                    })
                    .map(|(colliding_e, _)| colliding_e)
                    .collect();
                let obstructed_entities: HashSet<_> = colliding_entities
                    .difference(&unobstructed_entities)
                    .cloned()
                    .collect();
                unobstructed_entities.into_iter().for_each(|vision_target| {
                    cmds.emit_event(InteractReq::<VisionField>::new(vf_entity, vision_target))
                });
                obstructed_entities.into_iter().for_each(|vision_target| {
                    cmds.emit_event(UninteractReq::<VisionField>::new(vf_entity, vision_target))
                })
            });
    }
}
