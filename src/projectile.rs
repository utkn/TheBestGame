use std::marker::PhantomData;

use notan::egui::epaint::ahash::HashSet;

use crate::{
    activation::{Activatable, ActivatableComponent, ActivatedEvt},
    core::*,
    entity_insights::EntityInsights,
    equipment::Equipment,
    needs::NeedMutator,
    physics::{Hitbox, HitboxType, Shape},
    storage::Storage,
    timed::{TimedEmit, TimedRemove},
};

use rand::Rng;

#[derive(Clone, Debug)]
pub struct ProjectileDefn {
    pub lifetime: f32,
    pub speed: f32,
    pub spread: f32,
    pub on_hit: NeedMutator,
}

#[derive(Clone, Debug)]
pub struct ProjectileGenerator {
    pub proj: ProjectileDefn,
    pub cooldown: Option<f32>,
}

#[derive(Clone, Copy, Debug)]
pub struct GenerateProjectileReq(EntityRef);

impl ActivatableComponent for ProjectileGenerator {
    fn can_activate(
        actor: &EntityRef,
        target: &EntityRef,
        _proj_gen: &Self,
        state: &State,
    ) -> bool {
        state
            .select_one::<(Equipment,)>(actor)
            .map(|(actor_equipment,)| actor_equipment.contains(target))
            .unwrap_or(false)
    }

    fn activation_priority() -> usize {
        Storage::activation_priority() + 10
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProjectileGenerationSystem;

impl System for ProjectileGenerationSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // In response to projectile generator activation events, emit generate projectile request.
        state
            .read_events::<ActivatedEvt<ProjectileGenerator>>()
            .for_each(|evt| {
                cmds.emit_event(GenerateProjectileReq(evt.activatable));
            });
        // Handle the generate projectile requests.
        state
            .read_events::<GenerateProjectileReq>()
            .filter_map(|evt| {
                state
                    .select_one::<(
                        Activatable<ProjectileGenerator>,
                        ProjectileGenerator,
                        Transform,
                    )>(&evt.0)
                    .map(|c| (evt.0, c))
            })
            .for_each(|(p_gen_entity, (activatable, p_gen, trans))| {
                // Make sure that the generator is active.
                if !activatable.curr_state {
                    return;
                }
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
                let anchor_parent = EntityInsights::of(&p_gen_entity, state).anchor_parent;
                // Determine the friendly entities of the projectile, which are...
                // ... the generator itself
                let mut friendly_entities = vec![p_gen_entity];
                // ... and the anchor parent of the generator
                friendly_entities.extend(anchor_parent);
                // Create the projectile entity.
                cmds.create_from((
                    *trans,
                    vel,
                    Lifetime {
                        remaining_time: p_gen.proj.lifetime,
                    },
                    Hitbox(HitboxType::Ghost, Shape::Circle(5.)),
                    // Do not hit the anchor parent.
                    Projectile::new(friendly_entities),
                    SuicideOnHit,
                    ApplyOnHit::new(Some(0.), p_gen.proj.on_hit.clone()),
                ));
                // If cooldown was set, remove the activatable component until the cooldown ends.
                if let Some(cooldown_time) = p_gen.cooldown {
                    cmds.set_component(
                        &p_gen_entity,
                        TimedEmit::new(cooldown_time, GenerateProjectileReq(p_gen_entity)),
                    );
                }
            });
    }
}

/// Represents a projectile that can impact other entities.
#[derive(Clone, Debug)]
pub struct Projectile {
    /// Hitting these entities will do nothing.
    friendly_entities: HashSet<EntityRef>,
}

impl Projectile {
    pub fn new(exceptions: impl IntoIterator<Item = EntityRef>) -> Self {
        Self {
            friendly_entities: HashSet::from_iter(exceptions),
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
    }
}

/// Entities tagged with this component will be removed after they hit another component.
#[derive(Clone, Copy, Debug)]
pub struct SuicideOnHit;

/// A system that removes the [`SuicideOnHit`] entities from the system when they hit a concrete entity.
#[derive(Clone, Copy, Debug)]
pub struct SuicideOnHitSystem;

impl System for SuicideOnHitSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Remove the entities that should be removed after a hit.
        state.read_events::<ProjectileHitEvt>().for_each(|evt| {
            if let Some(_) = state.select_one::<(SuicideOnHit,)>(&evt.hitter) {
                cmds.remove_entity(&evt.hitter);
            }
        });
    }
}

/// The entities that are hit by this entity will be applied this component with an optional time.
#[derive(Clone, Copy, Debug)]
pub struct ApplyOnHit<T: Component> {
    time: Option<f32>,
    component: T,
}

impl<T: Component> ApplyOnHit<T> {
    pub fn new(time: Option<f32>, component_to_apply: T) -> Self {
        Self {
            time,
            component: component_to_apply,
        }
    }
}

/// A system that handles the entities that apply `T` to the entities they hit.
#[derive(Clone, Debug)]
pub struct ApplyOnHitSystem<T: Component>(PhantomData<T>);

impl<T: Component> Default for ApplyOnHitSystem<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Component> System for ApplyOnHitSystem<T> {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state.read_events::<ProjectileHitEvt>().for_each(|evt| {
            if let Some((apply_on_hit,)) = state.select_one::<(ApplyOnHit<T>,)>(&evt.hitter) {
                let target = evt.target;
                let component_to_apply = apply_on_hit.component.clone();
                // Add the component.
                cmds.set_component(&target, component_to_apply);
                // Optionally, request the removal of the said component after a certain time.
                if let Some(time) = apply_on_hit.time {
                    cmds.set_component(&target, TimedRemove::<T>::new(time));
                }
            }
        });
    }
}
