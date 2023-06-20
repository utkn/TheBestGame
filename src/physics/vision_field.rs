use std::collections::HashSet;

use itertools::Itertools;

use crate::{
    interaction::{InteractTarget, Interaction, TryInteractTargetedReq, TryUninteractTargetedReq},
    prelude::*,
};

use super::{CollisionEndEvt, CollisionState, EffectiveHitbox, Hitbox};

/// Entities tagged with this component will initiate interactions with the entities that collide and are visible from the position of this entity.
#[derive(Clone, Copy, Debug)]
pub struct VisionField(pub f32);

impl Interaction for VisionField {
    fn priority() -> usize {
        0
    }

    fn can_start(actor: &EntityRef, _target: &EntityRef, state: &State) -> bool {
        state.select_one::<(VisionField,)>(actor).is_some()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VisionSystem;

impl System for VisionSystem {
    fn update(&mut self, _ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state.read_events::<CollisionEndEvt>().for_each(|evt| {
            if let Some(_) = state.select_one::<(VisionField,)>(&evt.e1) {
                cmds.emit_event(TryUninteractTargetedReq::<VisionField>::new(evt.e1, evt.e2));
            }
        });
        state
            .select::<(VisionField, CollisionState, Transform)>()
            .for_each(|(vision_field, (_, coll_state, ref_trans))| {
                let ref_pos = notan::math::vec2(ref_trans.x, ref_trans.y);
                let vf_anchor_parent = state
                    .select_one::<(AnchorTransform,)>(&vision_field)
                    .map(|(anchor,)| anchor.0);
                let colliding_entities: HashSet<_> = coll_state
                    .colliding
                    .iter()
                    // Do not consider the anchor parent in the vision.
                    .filter(|colliding_e| {
                        let is_anchor_parent = vf_anchor_parent
                            .map(|anchor_parent| anchor_parent == **colliding_e)
                            .unwrap_or(false);
                        !is_anchor_parent
                    })
                    .filter(|colliding_e| {
                        state
                            .select_one::<(InteractTarget<VisionField>,)>(colliding_e)
                            .is_some()
                    })
                    .cloned()
                    .collect();
                let colliding_hitboxes = colliding_entities
                    .iter()
                    .filter_map(|colliding_e| {
                        state
                            .select_one::<(Hitbox, Transform)>(colliding_e)
                            .map(|(hb, trans)| {
                                (
                                    *colliding_e,
                                    trans,
                                    EffectiveHitbox::new(colliding_e, trans, hb),
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
                                    (ref_pos.x, ref_pos.y),
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
                    cmds.emit_event(TryInteractTargetedReq::<VisionField>::new(
                        vision_field,
                        vision_target,
                    ))
                });
                obstructed_entities.into_iter().for_each(|vision_target| {
                    cmds.emit_event(TryUninteractTargetedReq::<VisionField>::new(
                        vision_field,
                        vision_target,
                    ))
                })
            });
    }
}
