use crate::{
    physics::{Hitbox, HitboxType, Shape, VisionField},
    prelude::*,
};

pub struct BuildingBundle {
    wall_lft: EntityRef,
    wall_rgt: EntityRef,
    wall_top: EntityRef,
    wall_btm_lft: EntityRef,
    wall_btm_rgt: EntityRef,
    activator: EntityRef,
}

impl BuildingBundle {
    pub fn create(trans: Transform, width: f32, height: f32, cmds: &mut StateCommands) -> Self {
        let activator = cmds.create_from((
            trans,
            Hitbox(
                HitboxType::Ghost,
                Shape::Rect {
                    w: width,
                    h: height,
                },
            ),
            InteractTarget::<Hitbox>::default(),
        ));
        let wall_lft = cmds.create_from((
            Transform::default(),
            AnchorTransform(activator, (-width / 2., 0.)),
            InteractTarget::<VisionField>::default(),
            Hitbox(HitboxType::Static, Shape::Rect { w: 20., h: height }),
        ));
        let wall_rgt = cmds.create_from((
            Transform::default(),
            AnchorTransform(activator, (width / 2., 0.)),
            InteractTarget::<VisionField>::default(),
            Hitbox(HitboxType::Static, Shape::Rect { w: 20., h: height }),
        ));
        let wall_top = cmds.create_from((
            Transform::default(),
            AnchorTransform(activator, (0., -height / 2.)),
            InteractTarget::<VisionField>::default(),
            Hitbox(
                HitboxType::Static,
                Shape::Rect {
                    w: width - 10.,
                    h: 20.,
                },
            ),
        ));
        let wall_btm_lft = cmds.create_from((
            Transform::default(),
            AnchorTransform(activator, (-width / 4. - 10., height / 2.)),
            InteractTarget::<VisionField>::default(),
            Hitbox(
                HitboxType::Static,
                Shape::Rect {
                    w: width / 2. - 30.,
                    h: 20.,
                },
            ),
        ));
        let wall_btm_rgt = cmds.create_from((
            Transform::default(),
            AnchorTransform(activator, (width / 4. + 10., height / 2.)),
            InteractTarget::<VisionField>::default(),
            Hitbox(
                HitboxType::Static,
                Shape::Rect {
                    w: width / 2. - 30.,
                    h: 20.,
                },
            ),
        ));
        Self {
            wall_lft,
            wall_rgt,
            wall_top,
            wall_btm_lft,
            wall_btm_rgt,
            activator,
        }
    }
}
