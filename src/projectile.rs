use notan::egui::epaint::ahash::HashSet;

use crate::{
    activation::ActivatedEvt,
    core::*,
    entity_insights::EntityInsights,
    needs::{NeedMutator, NeedMutatorEffect, NeedMutatorTarget, NeedType},
    physics::{Hitbox, HitboxType, Shape},
};

#[derive(Clone, Debug)]
pub struct ProjectileDefn {
    pub lifetime: f32,
    pub speed: f32,
    pub need_mutation: (NeedType, NeedMutatorEffect),
}

#[derive(Clone, Debug)]
pub struct ProjectileGenerator(pub ProjectileDefn);

#[derive(Clone, Copy, Debug)]
pub struct ProjectileGenerationSystem;

impl System for ProjectileGenerationSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state.read_events::<ActivatedEvt>().for_each(|evt| {
            let p_gen_entity = evt.0;
            if let Some((p_gen, trans)) =
                state.select_one::<(ProjectileGenerator, Transform)>(&p_gen_entity)
            {
                // Compute the new velocity of the projectile.
                let vel = notan::math::Vec2::from_angle(trans.deg.to_radians()) * p_gen.0.speed;
                let vel = Velocity {
                    x: vel.x,
                    y: -vel.y, // y axis is inverted!
                };
                // Create the projectile entity.
                cmds.create_from((
                    *trans,
                    vel,
                    Lifetime {
                        remaining_time: p_gen.0.lifetime,
                    },
                    Hitbox(HitboxType::Ghost, Shape::Circle(5.)),
                    // Do not hit the anchor parent.
                    Projectile::new(EntityInsights::of(&p_gen_entity, state).anchor_parent, true),
                    NeedMutator::new(
                        [NeedMutatorTarget::HitTarget],
                        p_gen.0.need_mutation.0,
                        p_gen.0.need_mutation.1,
                    ),
                ));
            }
        });
    }
}

/// Represents a projectile that can impact other entities.
#[derive(Clone, Debug)]
pub struct Projectile {
    /// Hitting these entities will do nothing.
    friendly_entities: HashSet<EntityRef>,
    /// If set to true, the projectile is removed after a hit.
    remove_on_hit: bool,
}

impl Projectile {
    pub fn new(exceptions: impl IntoIterator<Item = EntityRef>, remove_on_hit: bool) -> Self {
        Self {
            friendly_entities: HashSet::from_iter(exceptions),
            remove_on_hit,
        }
    }
}

/// An event denoting a projectile hitting a concrete entity.
#[derive(Clone, Copy, Debug)]
pub struct ProjectileHitEvt {
    pub hitter: EntityRef,
    pub target: EntityRef,
}

#[derive(Clone, Copy, Debug)]
pub struct ProjectileHitSystem;

impl System for ProjectileHitSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Emit the projectile hit events.
        state.select::<(Projectile,)>().for_each(|(e, (hitter,))| {
            let hitter_insights = EntityInsights::of(&e, state);
            hitter_insights
                .new_collision_starters
                .into_iter()
                // Make sure that we do not consider friendly entities.
                .filter(|coll_target| !hitter.friendly_entities.contains(coll_target))
                // Make sure that the target's hitbox is concrete.
                .filter(|coll_target| {
                    let target_hb = state.select_one::<(Hitbox,)>(coll_target).unwrap().0;
                    target_hb.0.is_concrete()
                })
                .for_each(|coll_target| {
                    cmds.emit_event(ProjectileHitEvt {
                        hitter: e,
                        target: coll_target,
                    })
                });
        });
        // Remove the projectiles that should be removed after a hit.
        state.read_events::<ProjectileHitEvt>().for_each(|evt| {
            if let Some((hitter,)) = state.select_one::<(Projectile,)>(&evt.hitter) {
                if hitter.remove_on_hit {
                    cmds.remove_entity(&evt.hitter);
                }
            }
        });
    }
}
