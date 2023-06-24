use crate::{controller::ProximityInteractable, physics::*, prelude::*};

use super::Storage;

pub struct StorageBundle {
    storage: EntityRef,
    activator: EntityRef,
}

impl EntityBundle for StorageBundle {
    fn create(trans: Transform, cmds: &mut StateCommands) -> Self {
        let storage = cmds.create_from((
            trans,
            Hitbox(HitboxType::Static, Shape::Rect { w: 20., h: 20. }),
            InteractTarget::<Hitbox>::default(),
            InteractTarget::<Storage>::default(),
            Storage::new(60),
            InteractTarget::<VisionField>::default(),
        ));
        let activator = cmds.create_from((
            Transform::default(),
            AnchorTransform(storage, (0., 0.)),
            ProximityInteractable,
            UntargetedInteractionDelegate(storage),
            Hitbox(HitboxType::Ghost, Shape::Rect { w: 40., h: 40. }),
            InteractTarget::<Hitbox>::default(),
            ExistenceDependency(storage),
        ));
        Self { storage, activator }
    }

    fn primary_entity(&self) -> &EntityRef {
        &self.storage
    }

    fn try_reconstruct(storage: &EntityRef, state: &State) -> Option<Self> {
        let activator = state
            .select::<(
                ProximityInteractable,
                AnchorTransform,
                UntargetedInteractionDelegate,
            )>()
            .find(|(_, (_, anchor, intr_delegate))| {
                &anchor.0 == storage && &intr_delegate.0 == storage
            })
            .map(|(e, _)| e)?;
        Some(Self {
            storage: *storage,
            activator,
        })
    }
}
