use std::marker::PhantomData;

use crate::{
    character::CharacterInsights, item::*, needs::NeedMutator, physics::*, sprite::Sprite,
};

use rand::Rng;

/// Defines a projectile to be generated by [`ProjectileGenerator`]s.
#[derive(Clone, Debug)]
pub struct ProjectileDefn {
    pub lifetime: f32,
    pub speed: f32,
    pub spread: f32,
    pub on_hit: NeedMutator,
}

/// Entities tagged with this components will be able to generate projectiles upon interaction.
#[derive(Clone, Debug)]
pub struct ProjectileGenerator {
    pub proj: ProjectileDefn,
    pub cooldown: Option<f32>,
    pub auto_knockback: Option<f32>,
}

/// [`ProjectileGenerator`]s denote an interaction, which lets them shoot a projectile.
impl Interaction for ProjectileGenerator {
    fn priority() -> usize {
        Storage::priority() + 10
    }

    fn can_start_targeted(_actor: &EntityRef, _target: &EntityRef, _state: &State) -> bool {
        true
    }

    fn can_start_untargeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        let insights = StateInsights::of(state);
        insights.is_equipping(actor, target) && insights.is_character(actor)
    }

    fn can_end_untargeted(_actor: &EntityRef, _target: &EntityRef, _state: &State) -> bool {
        true
    }
}

/// A request to generate a projectile from the wrapped entity.
#[derive(Clone, Copy, Debug)]
pub struct GenerateProjectileReq {
    actor_entity: EntityRef,
    gen_entity: EntityRef,
}

/// A system that handles projectile generation by [`ProjectileGenerator`]s.
#[derive(Clone, Copy, Debug)]
pub struct ProjectileGenerationSystem;

impl System for ProjectileGenerationSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Try to automatically uninteract from the activated [`ProjectileGenerator`]s upon unequipping them.
        state.read_events::<ItemUnequippedEvt>().for_each(|evt| {
            if Storage::interaction_exists(&evt.equipment_entity, &evt.item_entity, state) {
                cmds.emit_event(UninteractReq::<ProjectileGenerator>::new(
                    evt.equipment_entity,
                    evt.item_entity,
                ));
            }
        });
        // In response to projectile generator activation events, emit generate projectile request.
        state
            .read_events::<InteractionStartedEvt<ProjectileGenerator>>()
            .for_each(|evt| {
                cmds.emit_event(GenerateProjectileReq {
                    actor_entity: evt.actor,
                    gen_entity: evt.target,
                });
            });
        // Handle the generate projectile requests.
        state
            .read_events::<GenerateProjectileReq>()
            .filter_map(|evt| {
                state
                    .select_one::<(
                        InteractTarget<ProjectileGenerator>,
                        ProjectileGenerator,
                        Transform,
                    )>(&evt.gen_entity)
                    .map(|c| (evt.actor_entity, evt.gen_entity, c))
            })
            .for_each(
                |(actor_entity, gen_entity, (interact_target, p_gen, trans))| {
                    // Make sure that the generator is active.
                    if interact_target.actors.len() == 0 {
                        return;
                    }
                    // Compute the new velocity of the projectile.
                    let rand_spread = if p_gen.proj.spread > 0. {
                        rand::thread_rng().gen_range(0.0..p_gen.proj.spread)
                            - p_gen.proj.spread / 2.
                    } else {
                        0.
                    };
                    let mut new_trans = trans.with_deg(trans.deg + rand_spread);
                    let dir = new_trans.dir_vec();
                    let dir = notan::math::vec2(dir.0, dir.1);
                    let new_pos = notan::math::vec2(trans.x, trans.y) + dir * 20.;
                    new_trans.x = new_pos.x;
                    new_trans.y = new_pos.y;
                    let vel = dir * p_gen.proj.speed;
                    let vel = Velocity { x: vel.x, y: vel.y };
                    let anchor_parent = StateInsights::of(state).anchor_parent_of(&gen_entity);
                    // Determine the friendly entities of the projectile, which are...
                    // ... the generator itself
                    let mut friendly_entities = vec![gen_entity];
                    // ... and the anchor parent of the generator
                    friendly_entities.extend(anchor_parent);
                    // Create the projectile entity.
                    cmds.create_from((
                        new_trans,
                        vel,
                        Lifetime {
                            remaining_time: p_gen.proj.lifetime,
                        },
                        Hitbox(HitboxType::Ghost, Shape::Circle { r: 5. }),
                        InteractTarget::<Hitbox>::default(),
                        // Do not hit the anchor parent.
                        Hitter::new(friendly_entities),
                        SuicideOnHit,
                        ApplyOnHit::new(Some(0.), p_gen.proj.on_hit.clone()),
                        Sprite::new("bullet", 2),
                    ));
                    // Apply knockback optionally
                    if let Some(knockback_factor) = p_gen.auto_knockback {
                        let knockback_vel = dir * -1. * knockback_factor;
                        cmds.update_component(&actor_entity, move |actor_vel: &mut Velocity| {
                            actor_vel.x += knockback_vel.x;
                            actor_vel.y += knockback_vel.y;
                        });
                    }
                    // If cooldown was set, remove the activatable component until the cooldown ends.
                    if let Some(cooldown_time) = p_gen.cooldown {
                        cmds.set_component(
                            &gen_entity,
                            TimedEmit::new(
                                cooldown_time,
                                GenerateProjectileReq {
                                    actor_entity,
                                    gen_entity,
                                },
                            ),
                        );
                    }
                },
            );
    }
}

/// Represents an entity that can impact other entities on collision.
#[derive(Clone, Debug)]
pub struct Hitter {
    /// Hitting these entities will do nothing.
    friendly_entities: HashSet<EntityRef>,
}

impl EntityRefBag for Hitter {
    fn remove_invalids(&mut self, entity_mgr: &EntityManager) {
        self.friendly_entities
            .iter()
            .filter(|e| !entity_mgr.is_valid(e))
            .cloned()
            .collect_vec()
            .into_iter()
            .for_each(|invalid| {
                self.friendly_entities.remove(&invalid);
            });
    }
}

impl Hitter {
    pub fn new(exceptions: impl IntoIterator<Item = EntityRef>) -> Self {
        Self {
            friendly_entities: HashSet::from_iter(exceptions),
        }
    }
}

/// An event denoting a [`Hitter`] hitting a concrete entity.
#[derive(Clone, Copy, Debug)]
pub struct HitEvt {
    pub hitter: EntityRef,
    pub target: EntityRef,
    pub hit_velocity: (f32, f32),
}

/// A system that listens to [`Hitter`] collision and emits appropriate [`HitEvt`]s.
#[derive(Clone, Copy, Debug)]
pub struct HitSystem;

impl System for HitSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Emit the projectile hit events.
        state
            .select::<(Hitter, Velocity)>()
            .for_each(|(hitter_entity, (hitter, hitter_vel))| {
                StateInsights::of(state)
                    .new_collision_starters_of(&hitter_entity)
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
                        cmds.emit_event(HitEvt {
                            hitter: hitter_entity,
                            target: *coll_target,
                            hit_velocity: (hitter_vel.x, hitter_vel.y),
                        })
                    });
            });
        state.select::<(Hitter,)>().for_each(|(e, _)| {
            cmds.remove_invalids::<Hitter>(&e);
        })
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
        state.read_events::<HitEvt>().for_each(|evt| {
            if let Some(_) = state.select_one::<(SuicideOnHit,)>(&evt.hitter) {
                cmds.mark_for_removal(&evt.hitter);
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
        state.read_events::<HitEvt>().for_each(|evt| {
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
