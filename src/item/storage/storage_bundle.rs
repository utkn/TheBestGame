use crate::{controller::ProximityInteractable, physics::*, prelude::*};

use super::Storage;

#[derive(Clone, Copy, Debug)]
pub struct StorageBundle {
    storage: EntityRef,
    activator: EntityRef,
}

impl StorageBundle {
    pub fn create(trans: Transform, cmds: &mut StateCommands) -> Self {
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
        cmds.push_bundle(Self { storage, activator })
    }
}

impl<'a> EntityBundle<'a> for StorageBundle {
    type TupleRepr = (EntityRef, EntityRef);

    fn primary_entity(&self) -> &EntityRef {
        &self.storage
    }

    fn deconstruct(self) -> Self::TupleRepr {
        (self.storage, self.activator)
    }

    fn reconstruct(args: <Self::TupleRepr as EntityTuple<'a>>::AsRefTuple) -> Self {
        Self {
            storage: *args.0,
            activator: *args.1,
        }
    }
}
