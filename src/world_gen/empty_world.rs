use crate::ai::*;
use crate::controller::*;
use crate::effects::*;
use crate::item::*;
use crate::needs::*;
use crate::physics::*;
use crate::prelude::*;

use crate::vehicle::*;

pub fn create_empty_world() -> World {
    // Create the world from an empty state.
    let mut world = World::from(State::default());
    // Control & movement
    world.register_system(MovementSystem);
    world.register_system(AnchorSystem);
    world.register_system(ControlSystem::<UserInputDriver>::default());
    world.register_system(LifetimeSystem);
    world.register_system(ApproachVelocitySystem);
    world.register_system(ApproachRotationSystem);
    // Interactions
    world.register_system(InteractionAcceptorSystem(
        ConsensusStrategy::MaxPriority,
        ConsensusStrategy::MinPriority,
    ));
    world.register_system(ProximityInteractionSystem);
    world.register_system(HandInteractionSystem);
    world.register_system(UntargetedInteractionDelegateSystem);
    // Basic physics
    world.register_system(CollisionDetectionSystem);
    world.register_system(SeparateCollisionsSystem);
    world.register_system(InteractionSystem::<Hitbox>::default());
    // Item stuff
    world.register_system(StorageSystem);
    world.register_system(EquipmentSystem);
    world.register_system(ItemTransferSystem);
    world.register_system(ItemAnchorSystem);
    world.register_system(ItemPickupSystem);
    world.register_system(InteractionSystem::<Item>::default());
    world.register_system(InteractionSystem::<Storage>::default());
    world.register_system(InteractionSystem::<Equipment>::default());
    // Needs
    world.register_system(NeedStateSystem::default());
    world.register_system(NeedMutatorSystem);
    // Projectiles
    world.register_system(InteractionSystem::<ProjectileGenerator>::default());
    world.register_system(ProjectileGenerationSystem);
    world.register_system(HitSystem);
    world.register_system(SuicideOnHitSystem);
    world.register_system(TimedEmitSystem::<GenerateProjectileReq>::default());
    world.register_system(ApplyOnHitSystem::<NeedMutator>::default());
    // Vehicle stuff
    world.register_system(VehicleSystem);
    world.register_system(InteractionSystem::<Vehicle>::default());
    // AI stuff
    world.register_system(VisionSystem);
    world.register_system(InteractionSystem::<VisionField>::default());
    // Misc
    world.register_system(TimedRemoveSystem::<NeedMutator>::default());
    world.register_system(EffectSystem::<MaxSpeed>::default());
    world.register_system(EffectSystem::<Acceleration>::default());
    world.register_system(ExistenceDependencySystem);
    world
}
