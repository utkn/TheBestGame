use std::collections::HashSet;

use crate::{
    physics::{Hitbox, HitboxType, Shape, VisionField},
    prelude::*,
    sprite::Sprite,
};

mod building_tags;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WallDirection {
    Left,
    Right,
    Top,
    Bottom,
}

impl WallDirection {
    pub fn all_set() -> HashSet<WallDirection> {
        HashSet::from_iter([Self::Left, Self::Right, Self::Top, Self::Bottom])
    }

    pub fn none_set() -> HashSet<WallDirection> {
        HashSet::new()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Building;

#[derive(Clone, Copy, Debug)]
pub struct BuildingBundle {
    wall_lft: EntityRef,
    wall_rgt: EntityRef,
    wall_top: EntityRef,
    wall_btm: EntityRef,
    building: EntityRef,
    floors: EntityRef,
}

impl BuildingBundle {
    pub fn create(
        trans: Transform,
        width: f32,
        height: f32,
        walls: HashSet<WallDirection>,
        doors: HashSet<WallDirection>,
        sprite_id: &'static str,
        cmds: &mut StateCommands,
    ) -> Self {
        assert!(width >= 0. && height >= 0.);
        let num_tiles = (width / 64.) as u8;
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
            Sprite::new(format!("{}/ceilings", sprite_id), 20).with_tiling(num_tiles, num_tiles),
        ));
        let floors = cmds.create_from((
            Transform::default(),
            AnchorTransform(building, (0., 0.), 0.),
            Hitbox(
                HitboxType::Ghost,
                Shape::Rect {
                    w: width,
                    h: height,
                },
            ),
            Sprite::new(format!("{}/floors", sprite_id), 0).with_tiling(num_tiles, num_tiles),
        ));
        let wall_lft = if walls.contains(&WallDirection::Left) {
            let &wall = WallBundle::create(
                Transform::default(),
                height,
                doors.contains(&WallDirection::Left),
                sprite_id,
                cmds,
            )
            .primary_entity();
            cmds.set_component(&wall, AnchorTransform(building, (-width / 2., 0.), 90.));
            wall
        } else {
            cmds.create_entity()
        };
        let wall_top = if walls.contains(&WallDirection::Top) {
            let &wall = WallBundle::create(
                Transform::default(),
                height,
                doors.contains(&WallDirection::Top),
                sprite_id,
                cmds,
            )
            .primary_entity();
            cmds.set_component(&wall, AnchorTransform(building, (0., -height / 2.), 0.));
            wall
        } else {
            cmds.create_entity()
        };
        let wall_rgt = if walls.contains(&WallDirection::Right) {
            let &wall = WallBundle::create(
                Transform::default(),
                height,
                doors.contains(&WallDirection::Right),
                sprite_id,
                cmds,
            )
            .primary_entity();
            cmds.set_component(&wall, AnchorTransform(building, (width / 2., 0.), 90.));
            wall
        } else {
            cmds.create_entity()
        };
        let wall_btm = if walls.contains(&WallDirection::Bottom) {
            let &wall = WallBundle::create(
                Transform::default(),
                height,
                doors.contains(&WallDirection::Bottom),
                sprite_id,
                cmds,
            )
            .primary_entity();
            cmds.set_component(&wall, AnchorTransform(building, (0., height / 2.), 0.));
            wall
        } else {
            cmds.create_entity()
        };
        cmds.push_bundle(Self {
            building,
            wall_lft,
            wall_rgt,
            wall_top,
            wall_btm,
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
            self.wall_btm,
            self.floors,
        )
    }

    fn reconstruct(args: <Self::TupleRepr as EntityTuple<'a>>::AsRefTuple) -> Self {
        Self {
            building: *args.0,
            wall_lft: *args.1,
            wall_top: *args.2,
            wall_rgt: *args.3,
            wall_btm: *args.4,
            floors: *args.5,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WallBundle {
    wall: EntityRef,
    left: EntityRef,
    right: EntityRef,
}

impl WallBundle {
    pub fn create(
        trans: Transform,
        size: f32,
        with_door: bool,
        sprite_id: &'static str,
        cmds: &mut StateCommands,
    ) -> Self {
        let num_tiles = if with_door {
            (size / 32. - 2.) as u8
        } else {
            (size / 32.) as u8
        };
        let door_offset = if with_door { 32. } else { 0. };
        let wall = cmds.create_from((trans,));
        let left = cmds.create_from((
            Transform::default(),
            AnchorTransform(wall, (-size / 4. - door_offset / 2., 0.), 0.),
            Hitbox(
                HitboxType::Static,
                Shape::Rect {
                    w: size / 2. - door_offset,
                    h: 20.,
                },
            ),
            InteractTarget::<Hitbox>::default(),
            InteractTarget::<VisionField>::default(),
            Sprite::new(format!("{}/walls", sprite_id), 19).with_tiling(num_tiles, 1),
        ));
        let right = cmds.create_from((
            Transform::default(),
            AnchorTransform(wall, (size / 4. + door_offset / 2., 0.), 0.),
            Hitbox(
                HitboxType::Static,
                Shape::Rect {
                    w: size / 2. - door_offset,
                    h: 20.,
                },
            ),
            InteractTarget::<Hitbox>::default(),
            InteractTarget::<VisionField>::default(),
            Sprite::new(format!("{}/walls", sprite_id), 19).with_tiling(num_tiles, 1),
        ));
        Self { wall, left, right }
    }
}

impl<'a> EntityBundle<'a> for WallBundle {
    type TupleRepr = (EntityRef, EntityRef, EntityRef);

    fn primary_entity(&self) -> &EntityRef {
        &self.wall
    }

    fn deconstruct(self) -> Self::TupleRepr {
        (self.wall, self.left, self.right)
    }

    fn reconstruct(args: <Self::TupleRepr as EntityTuple<'a>>::AsRefTuple) -> Self {
        Self {
            wall: *args.0,
            left: *args.1,
            right: *args.2,
        }
    }
}
