use std::collections::HashSet;

use itertools::Itertools;
use sepax2d::{sat_collision, sat_overlap};

use crate::core::*;

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Circle(f32),
    Rect(f32, f32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitboxType {
    Ghost,   // not concrete
    Static,  // concrete and static
    Dynamic, // concrete and dynamic
}

impl HitboxType {
    pub fn is_concrete(&self) -> bool {
        matches!(self, HitboxType::Static | HitboxType::Dynamic)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Hitbox(pub HitboxType, pub Shape);

#[derive(Clone, Default, Debug)]
pub struct CollisionState {
    pub colliding: EntityRefSet,
}

#[derive(Clone, Copy, Debug)]
pub struct CollisionEvt {
    pub e1: EntityRef,
    pub e2: EntityRef,
    pub overlap: (f32, f32),
}

#[derive(Clone, Copy, Debug)]
pub struct CollisionStartEvt {
    pub e1: EntityRef,
    pub e2: EntityRef,
}

#[derive(Clone, Copy, Debug)]
pub struct CollisionEndEvt {
    pub e1: EntityRef,
    pub e2: EntityRef,
}

/// Represents a transformed shape that can be directly checked against other `TransformedShape`s for collisions.
#[derive(Clone, Copy, Debug)]
pub enum TransformedShape {
    Circle(sepax2d::circle::Circle),
    AABB(sepax2d::aabb::AABB),
}

impl TransformedShape {
    pub fn new(pos: &Transform, primitive_shape: &Shape) -> Self {
        match primitive_shape {
            Shape::Circle(r) => Self::Circle(sepax2d::circle::Circle::new((pos.x, pos.y), *r)),
            Shape::Rect(w, h) => Self::AABB(sepax2d::aabb::AABB::new(
                (pos.x - w / 2., pos.y - h / 2.),
                *w,
                *h,
            )),
        }
    }

    pub fn shape_ref(&self) -> &dyn sepax2d::Shape {
        match self {
            Self::Circle(shape) => shape,
            Self::AABB(shape) => shape,
        }
    }
}

/// Represents a collider that should be checked against collisions.
#[derive(Clone, Debug)]
pub struct EffectiveHitbox {
    pub entity: EntityRef,
    pub hb: Hitbox,
    pub trans: Transform,
    pub shape: TransformedShape,
}

impl EffectiveHitbox {
    pub fn new(e: &EntityRef, trans: &Transform, hb: &Hitbox) -> Self {
        Self {
            entity: *e,
            hb: *hb,
            trans: *trans,
            shape: TransformedShape::new(trans, &hb.1),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CollisionResponse {
    e1: EntityRef,
    e2: EntityRef,
    overlap: (f32, f32),
}

/// A system that detects collisions. Emits `CollisionStartEvt`, `CollisionEndEvt`, and `CollisionEvt` events.
#[derive(Clone, Debug, Default)]
pub struct CollisionDetectionSystem {
    colliding_pairs: HashSet<(EntityRef, EntityRef)>,
}

impl CollisionDetectionSystem {
    fn resolve_collision(
        ehb1: &EffectiveHitbox,
        ehb2: &EffectiveHitbox,
    ) -> Option<CollisionResponse> {
        if !sat_overlap(ehb1.shape.shape_ref(), ehb2.shape.shape_ref()) {
            return None;
        }
        // Get the collision response.
        let (resp_x, resp_y) = sat_collision(ehb1.shape.shape_ref(), ehb2.shape.shape_ref());
        let resp = notan::math::vec2(resp_x, resp_y);
        Some(CollisionResponse {
            e1: ehb1.entity,
            e2: ehb2.entity,
            overlap: (resp.x, resp.y),
        })
    }
}

impl System for CollisionDetectionSystem {
    fn update(&mut self, _ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        let effective_hbs = state
            .select::<(Transform, Hitbox)>()
            .map(|(e, (pos, hitbox))| EffectiveHitbox::new(&e, pos, hitbox))
            .collect_vec();
        let resps = effective_hbs
            .into_iter()
            .tuple_combinations()
            .flat_map(|(ehb1, ehb2)| Self::resolve_collision(&ehb1, &ehb2))
            .collect_vec();
        // Emit the collision events.
        resps.iter().for_each(|resp| {
            cmds.emit_event(CollisionEvt {
                e1: resp.e1,
                e2: resp.e2,
                overlap: resp.overlap,
            });
            cmds.emit_event(CollisionEvt {
                e1: resp.e2,
                e2: resp.e1,
                overlap: (-resp.overlap.0, -resp.overlap.1),
            });
        });
        // Generate the new pair of collisions.
        let colliding_pairs: HashSet<_> = resps
            .iter()
            .flat_map(|resp| [(resp.e1, resp.e2), (resp.e2, resp.e1)])
            .collect();
        // Detect the newly started collisions.
        let new_collisions: HashSet<_> = colliding_pairs
            .difference(&self.colliding_pairs)
            .cloned()
            .collect();
        // Detect the removed collisions.
        let mut old_collisions: HashSet<_> = self
            .colliding_pairs
            .difference(&colliding_pairs)
            .cloned()
            .collect();
        // Removed collisions should also contain the invalidated entities.
        self.colliding_pairs.iter().for_each(|(e1, e2)| {
            if !state.is_valid(e1) {
                old_collisions.insert((*e1, *e2));
            }
        });
        // Emit the appropriate events & update the staet.
        new_collisions.into_iter().for_each(|(e1, e2)| {
            cmds.emit_event(CollisionStartEvt { e1, e2 });
            cmds.update_component(&e1, move |c: &mut CollisionState| {
                c.colliding.insert(e2);
            });
        });
        old_collisions.into_iter().for_each(|(e1, e2)| {
            cmds.emit_event(CollisionEndEvt { e1, e2 });
            cmds.update_component(&e1, move |c: &mut CollisionState| {
                c.colliding.try_remove(&e2);
            });
        });
        // Update the collisions.
        self.colliding_pairs = colliding_pairs;
    }
}

/// A system that separates the colliding entities by listening to the `CollisionEvt` events.
#[derive(Clone, Debug, Default)]
pub struct SeparateCollisionsSystem;

impl System for SeparateCollisionsSystem {
    fn update(&mut self, _ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .read_events::<CollisionEvt>()
            .filter(|evt| {
                let anchored = state.select_one::<(AnchorTransform,)>(&evt.e1).is_some()
                    || state
                        .select_one::<(AnchorTransform,)>(&evt.e2)
                        .map(|(anchor,)| anchor.0 == evt.e1)
                        .unwrap_or(false);
                !anchored
            })
            .for_each(|evt| {
                if let Some((pos, hb)) = state.select_one::<(Transform, Hitbox)>(&evt.e1) {
                    let mut dpos = notan::math::vec2(-evt.overlap.0, -evt.overlap.1);
                    if dpos.length_squared() == 0. {
                        return;
                    }
                    if let Some((other_hb,)) = state.select_one::<(Hitbox,)>(&evt.e2) {
                        if hb.0 != HitboxType::Dynamic || other_hb.0 == HitboxType::Ghost {
                            return;
                        }
                        // this = dynamic, other = dynamic | static
                        if other_hb.0 == HitboxType::Dynamic {
                            dpos *= 0.5;
                        }
                        let new_pos = notan::math::vec2(pos.x, pos.y) + dpos;
                        cmds.set_component(&evt.e1, Transform::at(new_pos.x, new_pos.y));
                        // Reset the velocity.
                        if other_hb.0 == HitboxType::Static {
                            cmds.set_component(&evt.e1, Velocity::default());
                        }
                    }
                }
            })
    }
}
