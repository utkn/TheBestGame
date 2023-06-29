use std::collections::HashSet;

use itertools::Itertools;
use sepax2d::{sat_collision, sat_overlap, Rotate};

use crate::{camera::CameraFollow, prelude::*};

pub use collider_insights::*;
pub use projectile::*;
pub use projectile_insights::*;
pub use vision_field::*;
pub use vision_insights::*;

mod collider_insights;
mod projectile;
mod projectile_insights;
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

    fn can_start_targeted(_actor: &EntityRef, _target: &EntityRef, _state: &impl StateReader) -> bool {
        true
    }

    fn can_start_untargeted(_actor: &EntityRef, _target: &EntityRef, _state: &impl StateReader) -> bool {
        false
    }

    fn can_end_untargeted(_actor: &EntityRef, _target: &EntityRef, _state: &impl StateReader) -> bool {
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
    /// Creates a new transformed shape from the given transform, shape and offset (from the given transform)
    pub fn new(trans: &Transform, primitive_shape: &Shape) -> Self {
        match primitive_shape {
            Shape::Circle { r } => {
                Self::Circle(sepax2d::circle::Circle::new((trans.x, trans.y), *r))
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
                poly.rotate(-trans.deg.to_radians());
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
pub struct EffectiveHitbox<'a> {
    pub entity: EntityRef,
    pub hitbox: &'a Hitbox,
    pub shape: TransformedShape,
}

impl<'a> EffectiveHitbox<'a> {
    pub fn new(e: &EntityRef, state: &'a impl StateReader) -> Option<Self> {
        let (hitbox, trans) = state.select_one::<(Hitbox, Transform)>(e)?;
        Some(Self {
            entity: *e,
            hitbox,
            shape: TransformedShape::new(trans, &hitbox.1),
        })
    }

    pub fn new_speculative(e: &EntityRef, dt: f32, state: &'a impl StateReader) -> Option<Self> {
        let (hitbox, &(mut trans), vel) = state.select_one::<(Hitbox, Transform, Velocity)>(e)?;
        trans.x += vel.x * dt;
        trans.y += vel.y * dt;
        Some(Self {
            entity: *e,
            hitbox,
            shape: TransformedShape::new(&trans, &hitbox.1),
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

    fn separate_collision(
        e1: &EntityRef,
        e2: &EntityRef,
        overlap: &(f32, f32),
        state: &impl StateReader,
        cmds: &mut StateCommands,
    ) {
        let anchored = StateInsights::of(state).anchor_parent_of(e1).is_some()
            || StateInsights::of(state)
                .anchor_parent_of(&e2)
                .map(|parent| parent == e1)
                .unwrap_or(false);
        if anchored {
            return;
        }
        if let Some((hb,)) = state.select_one::<(Hitbox,)>(e1) {
            let mut dpos = notan::math::vec2(-overlap.0, -overlap.1);
            // who cares ??
            if dpos.length_squared() <= 1. {
                return;
            }
            if let Some((other_hb,)) = state.select_one::<(Hitbox,)>(e2) {
                if hb.0 != HitboxType::Dynamic || other_hb.0 == HitboxType::Ghost {
                    return;
                }
                // this = dynamic, other = dynamic | static
                if other_hb.0 == HitboxType::Dynamic {
                    dpos *= 0.5;
                }
                cmds.update_component(e1, move |trans: &mut Transform| {
                    trans.x += dpos.x;
                    trans.y += dpos.y;
                });
                // Reset the velocity.
                if other_hb.0 == HitboxType::Static {
                    cmds.set_component(e1, Velocity::default());
                    // cmds.set_component(e1, TargetVelocity::default());
                }
            }
        }
    }
}

impl<R: StateReader> System<R> for CollisionDetectionSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &R, cmds: &mut StateCommands) {
        let collision_bounds = state
            .select::<(Transform, CameraFollow)>()
            .next()
            .map(|(_, (trans, camera))| {
                (
                    (trans.x - camera.w / 2., trans.y - camera.h / 2.),
                    (trans.x + camera.w / 2., trans.y + camera.h / 2.),
                )
            })
            .unwrap_or_default();
        let effective_hbs = state
            .select::<(Transform, Hitbox)>()
            .filter(|(_, (trans, _))| {
                trans.x.clamp(collision_bounds.0 .0, collision_bounds.1 .0) == trans.x
                    && trans.y.clamp(collision_bounds.0 .1, collision_bounds.1 .1) == trans.y
            })
            .flat_map(|(e, _)| {
                if let Some(_) = state.select_one::<(Velocity,)>(&e) {
                    EffectiveHitbox::new_speculative(&e, ctx.dt, state)
                } else {
                    EffectiveHitbox::new(&e, state)
                }
            })
            .collect_vec();
        let resps = effective_hbs
            .into_iter()
            .tuple_combinations()
            .filter(|(ehb1, _)| ehb1.hitbox.0 != HitboxType::Static)
            .flat_map(|(ehb1, ehb2)| Self::resolve_collision(&ehb1, &ehb2))
            .collect_vec();
        // Separate the colliding pairs.
        resps.iter().for_each(|resp| {
            let neg_overlap = (-resp.overlap.0, -resp.overlap.1);
            Self::separate_collision(&resp.e1, &resp.e2, &resp.overlap, state, cmds);
            Self::separate_collision(&resp.e2, &resp.e1, &neg_overlap, state, cmds);
        });
        // Generate the new pair of collisions.
        let colliding_pairs: HashSet<_> = resps.iter().map(|resp| (resp.e1, resp.e2)).collect();
        // Handle the hitbox interaction.
        state
            .select::<(InteractTarget<Hitbox>,)>()
            .filter(|(_, (hb_intr,))| hb_intr.actors.len() > 0)
            .flat_map(|(target, (hb_intr,))| {
                hb_intr.actors.iter().map(move |actor| (*actor, target))
            })
            .unique()
            .for_each(|(actor, target)| {
                if !colliding_pairs.contains(&(actor, target))
                    && !colliding_pairs.contains(&(target, actor))
                {
                    cmds.emit_event(UninteractReq::<Hitbox>::new(actor, target));
                }
            });
        colliding_pairs.into_iter().for_each(|(e1, e2)| {
            if !Hitbox::interaction_exists(&e1, &e2, state) {
                cmds.emit_event(InteractReq::<Hitbox>::new(e1, e2));
                cmds.emit_event(InteractReq::<Hitbox>::new(e2, e1));
            }
        });
    }
}
