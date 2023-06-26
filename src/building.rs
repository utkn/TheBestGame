use crate::{
    physics::{Hitbox, HitboxType, Shape, VisionField},
    prelude::*,
    sprite::{Sprite, TilingConfig},
};

mod building_tags;

#[derive(Clone, Copy, Debug)]
pub struct Building;

#[derive(Clone, Copy, Debug)]
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
        let building = cmds.create_from((
            trans,
            Hitbox(
                HitboxType::Ghost,
                Shape::Rect {
                    w: width,
                    h: height + 40.,
                },
            ),
            InteractTarget::<Hitbox>::default(),
            Sprite::new("derelict_house", 5).with_tiling(TilingConfig {
                repeat_x: Some(4),
                repeat_y: Some(4),
            }),
        ));
        let wall_lft = cmds.create_from((
            Transform::default(),
            AnchorTransform(building, (-width / 2., 0.)),
            InteractTarget::<VisionField>::default(),
            Hitbox(HitboxType::Static, Shape::Rect { w: 20., h: height }),
        ));
        let wall_rgt = cmds.create_from((
            Transform::default(),
            AnchorTransform(building, (width / 2., 0.)),
            InteractTarget::<VisionField>::default(),
            Hitbox(HitboxType::Static, Shape::Rect { w: 20., h: height }),
        ));
        let wall_top = cmds.create_from((
            Transform::default(),
            AnchorTransform(building, (0., -height / 2.)),
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
            AnchorTransform(building, (-width / 4. - 10., height / 2.)),
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
            AnchorTransform(building, (width / 4. + 10., height / 2.)),
            InteractTarget::<VisionField>::default(),
            Hitbox(
                HitboxType::Static,
                Shape::Rect {
                    w: width / 2. - 30.,
                    h: 20.,
                },
            ),
        ));
        cmds.push_bundle(Self {
            wall_lft,
            wall_rgt,
            wall_top,
            wall_btm_lft,
            wall_btm_rgt,
            activator: building,
        })
    }
}

impl<'a> EntityBundle<'a> for BuildingBundle {
    type TupleRepr = (
        EntityRef,
        EntityRef,
        EntityRef,
        EntityRef,
        EntityRef,
        EntityRef,
    );

    fn primary_entity(&self) -> &EntityRef {
        &self.activator
    }

    fn deconstruct(self) -> Self::TupleRepr {
        (
            self.activator,
            self.wall_lft,
            self.wall_top,
            self.wall_rgt,
            self.wall_btm_rgt,
            self.wall_btm_lft,
        )
    }

    fn reconstruct(args: <Self::TupleRepr as EntityTuple<'a>>::AsRefTuple) -> Self {
        Self {
            activator: *args.0,
            wall_lft: *args.1,
            wall_top: *args.2,
            wall_rgt: *args.3,
            wall_btm_rgt: *args.4,
            wall_btm_lft: *args.5,
        }
    }
}
