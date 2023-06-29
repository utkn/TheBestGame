use crate::ai::*;
use crate::controller::*;
use crate::effects::*;
use crate::item::*;
use crate::needs::*;
use crate::physics::*;
use crate::prelude::*;

use crate::sprite::SpriteAnimationSystem;
use crate::vehicle::*;

pub fn create_empty_world<R: StateReader>() -> SystemManager<R> {
    // Create the world from an empty state.
    let mut system_manager = SystemManager::from(R::default());
    // Control & movement
    system_manager.register_system(MovementSystem);
    system_manager.register_system(AnchorSystem);
    system_manager.register_system(ControlSystem::<UserInputDriver>::default());
    system_manager.register_system(LifetimeSystem);
    system_manager.register_system(ApproachVelocitySystem);
    system_manager.register_system(ApproachRotationSystem);
    // Interactions
    system_manager.register_system(InteractionAcceptorSystem(
        ConsensusStrategy::MaxPriority, // start order
        ConsensusStrategy::MinPriority, // end order
    ));
    system_manager.register_system(ProximityInteractionSystem);
    system_manager.register_system(EquipmentInteractionSystem);
    system_manager.register_system(UntargetedInteractionDelegateSystem);
    // Basic physics
    system_manager.register_system(CollisionDetectionSystem);
    system_manager.register_system(InteractionSystem::<Hitbox>::default());
    // Item stuff
    system_manager.register_system(StorageSystem);
    system_manager.register_system(EquipmentSystem);
    system_manager.register_system(ItemTransferSystem);
    system_manager.register_system(ItemPickupSystem);
    system_manager.register_system(StorageDeactivationSystem);
    system_manager.register_system(InteractionSystem::<Item>::default());
    system_manager.register_system(InteractionSystem::<Storage>::default());
    system_manager.register_system(InteractionSystem::<Equipment>::default());
    // Needs
    system_manager.register_system(NeedStateSystem);
    system_manager.register_system(NeedMutatorSystem);
    // Projectiles
    system_manager.register_system(InteractionSystem::<ProjectileGenerator>::default());
    system_manager.register_system(ProjectileGenerationSystem);
    system_manager.register_system(HitSystem);
    system_manager.register_system(SuicideOnHitSystem);
    system_manager.register_system(TimedEmitSystem::<GenerateProjectileReq>::default());
    system_manager.register_system(ApplyOnHitSystem::<NeedMutator>::default());
    // Vehicle stuff
    system_manager.register_system(VehicleSystem);
    system_manager.register_system(InteractionSystem::<Vehicle>::default());
    // AI stuff
    system_manager.register_system(VisionSystem);
    system_manager.register_system(InteractionSystem::<VisionField>::default());
    system_manager.register_system(ControlSystem::<AiDriver>::default());
    // Sprite
    system_manager.register_system(SpriteAnimationSystem);
    // Misc
    system_manager.register_system(TimedRemoveSystem::<NeedMutator>::default());
    system_manager.register_system(EffectSystem::<MaxSpeed>::default());
    system_manager.register_system(EffectSystem::<Acceleration>::default());
    system_manager
}
