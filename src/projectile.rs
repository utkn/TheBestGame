use notan::egui::epaint::ahash::HashSet;

use crate::{
    activation::Activatable,
    cooldown::Cooldown,
    core::*,
    entity_insights::EntityInsights,
    needs::{NeedMutator, NeedMutatorEffect, NeedMutatorTarget, NeedType},
    physics::{Hitbox, HitboxType, Shape},
};

use rand::Rng;

#[derive(Clone, Debug)]
pub struct ProjectileDefn {
    pub lifetime: f32,
    pub speed: f32,
    pub spread: f32,
    pub need_mutation: (NeedType, NeedMutatorEffect),
}

#[derive(Clone, Debug)]
pub struct ProjectileGenerator {
    pub proj: ProjectileDefn,
    pub cooldown: Option<f32>,
}

#[derive(Clone, Copy, Debug)]
pub struct ProjectileGenerationSystem;

impl System for ProjectileGenerationSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(Activatable, ProjectileGenerator, Transform)>()
            .filter(|(_, (activatable, _, _))| activatable.curr_state)
            .for_each(|(p_gen_entity, (_, p_gen, trans))| {
                // Compute the new velocity of the projectile.
                let rand_spread = if p_gen.proj.spread > 0. {
                    rand::thread_rng().gen_range(0.0..p_gen.proj.spread) - p_gen.proj.spread / 2.
                } else {
                    0.
                };
                let angles = trans.deg + rand_spread;
                let vel = notan::math::Vec2::from_angle(angles.to_radians()) * p_gen.proj.speed;
                let vel = Velocity {
                    x: vel.x,
                    y: -vel.y, // y axis is inverted!
                };
                // Determine the friendly entities of the projectile, which are...
                // ... the generator itself
                let mut friendly_entities = vec![p_gen_entity];
                // ... and the anchor parent of the generator
                friendly_entities.extend(EntityInsights::of(&p_gen_entity, state).anchor_parent);
                // Create the projectile entity.
                cmds.create_from((
                    *trans,
                    vel,
                    Lifetime {
                        remaining_time: p_gen.proj.lifetime,
                    },
                    Hitbox(HitboxType::Ghost, Shape::Circle(5.)),
                    // Do not hit the anchor parent.
                    Projectile::new(friendly_entities, true),
                    NeedMutator::new(
                        [NeedMutatorTarget::HitTarget],
                        p_gen.proj.need_mutation.0,
                        p_gen.proj.need_mutation.1,
                    ),
                ));
                // If cooldown was set, remove the activatable component until the cooldown ends.
                if let Some(cooldown_time) = p_gen.cooldown {
                    if let Some((old_activatable,)) =
                        state.select_one::<(Activatable,)>(&p_gen_entity)
                    {
                        cmds.remove_component::<Activatable>(&p_gen_entity);
                        cmds.set_component(
                            &p_gen_entity,
                            Cooldown::new(cooldown_time, old_activatable.clone().deactivated()),
                        )
                    }
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
                    state
                        .select_one::<(Hitbox,)>(coll_target)
                        .map(|(hb,)| hb.0.is_concrete())
                        .unwrap_or(false)
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
