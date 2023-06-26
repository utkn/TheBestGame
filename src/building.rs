use crate::{
    physics::{Hitbox, HitboxType, Shape, VisionField},
    prelude::*,
    sprite::Sprite,
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
    building: EntityRef,
    floors: EntityRef,
}

impl BuildingBundle {
    pub fn create(trans: Transform, width: f32, height: f32, cmds: &mut StateCommands) -> Self {
        let building = cmds.create_from((
            trans,
            Hitbox(
                HitboxType::Ghost,
                Shape::Rect {
                    w: width,
                    h: height,
                },
            ),
            InteractTarget::<Hitbox>::default(),
            Sprite::new("derelict_house/ceilings", 20).with_tiling(4, 4),
        ));
        let floors = cmds.create_from((
            Transform::default(),
            AnchorTransform(building, (0., 0.)),
            Hitbox(
                HitboxType::Ghost,
                Shape::Rect {
                    w: width,
                    h: height,
                },
            ),
            Sprite::new("derelict_house/floors", 0).with_tiling(4, 4),
        ));
        let wall_lft = cmds.create_from((
            Transform::default(),
            AnchorTransform(building, (-width / 2., 0.)),
            InteractTarget::<VisionField>::default(),
            Hitbox(HitboxType::Static, Shape::Rect { w: 20., h: height }),
            Sprite::new("derelict_house/walls", 19).with_tiling(1, 17),
        ));
        let wall_rgt = cmds.create_from((
            Transform::default(),
            AnchorTransform(building, (width / 2., 0.)),
            InteractTarget::<VisionField>::default(),
            Hitbox(HitboxType::Static, Shape::Rect { w: 20., h: height }),
            Sprite::new("derelict_house/walls", 19).with_tiling(1, 17),
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
            Sprite::new("derelict_house/walls", 19).with_tiling(17, 1),
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
            Sprite::new("derelict_house/walls", 19).with_tiling(7, 1),
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
            Sprite::new("derelict_house/walls", 19).with_tiling(7, 1),
        ));
        cmds.push_bundle(Self {
            building,
            wall_lft,
            wall_rgt,
            wall_top,
            wall_btm_lft,
            wall_btm_rgt,
            floors,
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
        EntityRef,
    );

    fn primary_entity(&self) -> &EntityRef {
        &self.building
    }

    fn deconstruct(self) -> Self::TupleRepr {
        (
            self.building,
            self.wall_lft,
            self.wall_top,
            self.wall_rgt,
            self.wall_btm_rgt,
            self.wall_btm_lft,
            self.floors,
        )
    }

    fn reconstruct(args: <Self::TupleRepr as EntityTuple<'a>>::AsRefTuple) -> Self {
        Self {
            building: *args.0,
            wall_lft: *args.1,
            wall_top: *args.2,
            wall_rgt: *args.3,
            wall_btm_rgt: *args.4,
            wall_btm_lft: *args.5,
            floors: *args.6,
        }
    }
}
