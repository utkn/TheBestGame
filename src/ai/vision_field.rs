use std::collections::HashSet;

use itertools::Itertools;

use crate::{physics::*, prelude::*};

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
                    cmds.emit_event(UninteractReq::<VisionField>::new(e, target));
                });
        });
        state.select::<(VisionField, Transform)>().for_each(
            |(vision_field_entity, (_, ref_trans))| {
                let ref_pos = notan::math::vec2(ref_trans.x, ref_trans.y);
                let vf_anchor_parent =
                    StateInsights::of(state).anchor_parent_of(&vision_field_entity);
                let colliding_entities: HashSet<_> = StateInsights::of(state)
                    .contacts_of(&vision_field_entity)
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
                    cmds.emit_event(InteractReq::<VisionField>::new(
                        vision_field_entity,
                        vision_target,
                    ))
                });
                obstructed_entities.into_iter().for_each(|vision_target| {
                    cmds.emit_event(UninteractReq::<VisionField>::new(
                        vision_field_entity,
                        vision_target,
                    ))
                })
            },
        );
    }
}
