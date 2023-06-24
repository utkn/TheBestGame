use std::collections::HashSet;

use itertools::Itertools;
use sepax2d::{sat_collision, sat_overlap, Rotate};

use crate::prelude::*;

pub use collider_insights::*;
pub use projectile::*;
pub use vision_field::*;
pub use vision_insights::*;

mod collider_insights;
mod projectile;
mod vision_field;
mod vision_insights;

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Circle { r: f32 },
    Rect { w: f32, h: f32 },
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

impl Interaction for Hitbox {
    fn priority() -> usize {
        0
    }

    fn can_start_targeted(_actor: &EntityRef, _target: &EntityRef, _state: &State) -> bool {
        true
    }

    fn can_start_untargeted(_actor: &EntityRef, _target: &EntityRef, _state: &State) -> bool {
        false
    }

    fn can_end_untargeted(_actor: &EntityRef, _target: &EntityRef, _state: &State) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CollisionEvt {
    pub e1: EntityRef,
    pub e2: EntityRef,
    pub overlap: (f32, f32),
}

/// Represents a transformed shape that can be directly checked against other `TransformedShape`s for collisions.
#[derive(Clone, Debug)]
pub enum TransformedShape {
    Circle(sepax2d::circle::Circle),
    Poly(sepax2d::polygon::Polygon),
}

impl TransformedShape {
    pub fn new(trans: &Transform, primitive_shape: &Shape, offset: (f32, f32)) -> Self {
        match primitive_shape {
            Shape::Circle { r } => {
                let rotated_offset = notan::math::Vec2::from_angle(-trans.deg.to_radians())
                    .rotate(notan::math::vec2(offset.0, offset.1));
                Self::Circle(sepax2d::circle::Circle::new(
                    (
                        trans.x - offset.0 + rotated_offset.x,
                        trans.y - offset.1 + rotated_offset.y,
                    ),
                    *r,
                ))
            }
            Shape::Rect { w, h } => {
                let mut poly = sepax2d::polygon::Polygon::from_vertices(
                    (0., 0.),
                    vec![
                        (-w / 2., -h / 2.),
                        (w / 2., -h / 2.),
                        (w / 2., h / 2.),
                        (-w / 2., h / 2.),
                    ],
                );
                poly.vertices.iter_mut().for_each(|v| {
                    v.0 += offset.0;
                    v.1 += offset.1;
                });
                poly.rotate(-trans.deg.to_radians());
                poly.vertices.iter_mut().for_each(|v| {
                    v.0 -= offset.0;
                    v.1 -= offset.1;
                });
                poly.position = (trans.x, trans.y);
                Self::Poly(poly)
            }
        }
    }

    pub fn shape_ref(&self) -> &dyn sepax2d::Shape {
        match self {
            Self::Circle(shape) => shape,
            Self::Poly(shape) => shape,
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
    pub fn new(e: &EntityRef, state: &State) -> Option<Self> {
        let (hb, trans) = state.select_one::<(Hitbox, Transform)>(e)?;
        let offset = state
            .select_one::<(AnchorTransform,)>(e)
            .map(|(anchor_trans,)| anchor_trans.1)
            .unwrap_or_default();
        Some(Self {
            entity: *e,
            hb: *hb,
            trans: *trans,
            shape: TransformedShape::new(trans, &hb.1, offset),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CollisionResponse {
    e1: EntityRef,
    e2: EntityRef,
    overlap: (f32, f32),
}

/// A system that detects collisions. Emits `CollisionEvt` events.
#[derive(Clone, Debug)]
pub struct CollisionDetectionSystem;

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
            .flat_map(|(e, _)| EffectiveHitbox::new(&e, state))
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
        let colliding_pairs: HashSet<_> = resps.iter().map(|resp| (resp.e1, resp.e2)).collect();
        let all_pairs: HashSet<_> = state
            .select_all()
            .collect_vec()
            .into_iter()
            .tuple_combinations()
            .collect();
        all_pairs.into_iter().for_each(|(e1, e2)| {
            if colliding_pairs.contains(&(e1, e2)) || colliding_pairs.contains(&(e2, e1)) {
                if !Hitbox::interaction_exists(&e1, &e2, state) {
                    cmds.emit_event(InteractReq::<Hitbox>::new(e1, e2));
                    cmds.emit_event(InteractReq::<Hitbox>::new(e2, e1));
                }
            } else if Hitbox::interaction_exists(&e1, &e2, state) {
                cmds.emit_event(UninteractReq::<Hitbox>::new(e1, e2));
                cmds.emit_event(UninteractReq::<Hitbox>::new(e2, e1));
            }
        });
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
                let anchored = StateInsights::of(state).anchor_parent_of(&evt.e1).is_some()
                    || StateInsights::of(state)
                        .anchor_parent_of(&evt.e2)
                        .map(|parent| parent == evt.e1)
                        .unwrap_or(false);
                !anchored
            })
            .for_each(|evt| {
                if let Some((trans, hb)) = state.select_one::<(Transform, Hitbox)>(&evt.e1) {
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
                        let new_pos = notan::math::vec2(trans.x, trans.y) + dpos;
                        cmds.set_component(&evt.e1, trans.with_pos(new_pos.x, new_pos.y));
                        // Reset the velocity.
                        if other_hb.0 == HitboxType::Static {
                            cmds.set_component(&evt.e1, Velocity::default());
                        }
                    }
                }
            })
    }
}
