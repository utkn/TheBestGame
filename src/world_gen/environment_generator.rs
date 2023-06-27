use std::collections::HashSet;

use itertools::Itertools;
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::{
    building::{BuildingBundle, WallDirection},
    prelude::*,
};

use super::Rect;

pub trait EnvGenerator {
    fn try_generate(
        self,
        available_space: &Rect,
        state: &State,
        cmds: &mut StateCommands,
    ) -> Option<EntityRef>;
}

pub struct HouseGenerator {
    pub sprite_id: &'static str,
}

impl HouseGenerator {
    pub fn new(sprite_id: &'static str) -> Self {
        Self { sprite_id }
    }
}

impl EnvGenerator for HouseGenerator {
    fn try_generate(
        self,
        available_space: &Rect,
        state: &State,
        cmds: &mut StateCommands,
    ) -> Option<EntityRef> {
        let mut rng = thread_rng();
        let possible_sizes = [192., 256.];
        let mut curr_pos = available_space.min;
        let mut available_x = available_space.w();
        let mut available_y = available_space.h();
        let mut rooms_to_generate = Vec::new();
        loop {
            let chosen_size_x = {
                let available_sizes = possible_sizes
                    .iter()
                    .filter(|size| *size <= &available_x)
                    .collect_vec();
                available_sizes.choose(&mut rng).cloned()
            };
            let chosen_size_y = {
                let available_sizes = possible_sizes
                    .iter()
                    .filter(|size| *size <= &available_y)
                    .collect_vec();
                available_sizes.choose(&mut rng).cloned()
            };
            let chosen_direction = match (chosen_size_x, chosen_size_y) {
                (None, None) => None,
                (None, Some(dy)) => Some((dy, WallDirection::Bottom)),
                (Some(dx), None) => Some((dx, WallDirection::Right)),
                (Some(dx), Some(dy)) => {
                    if thread_rng().gen() {
                        Some((dy, WallDirection::Bottom))
                    } else {
                        Some((dx, WallDirection::Right))
                    }
                }
            };
            let room_to_generate = chosen_direction.map(|(delta, dir)| {
                let delta = *delta;
                let room_area = Rect::new(curr_pos, (curr_pos.0 + delta, curr_pos.1 + delta));
                // update
                match dir {
                    WallDirection::Right => {
                        curr_pos.0 += delta;
                        available_x -= delta;
                    }
                    WallDirection::Bottom => {
                        curr_pos.1 += delta;
                        available_y -= delta
                    }
                    _ => unreachable!(),
                }
                (
                    room_area,
                    WallDirection::all_set(),
                    WallDirection::none_set(),
                    dir,
                )
            });
            if let Some(generated_room) = room_to_generate {
                rooms_to_generate.push(generated_room);
            } else {
                break;
            }
        }
        // Fix up the walls & doors.
        for j in 1..rooms_to_generate.len() {
            let i = j - 1;
            let (r1, mut walls_1, mut doors_1, dir_1) = rooms_to_generate.remove(i);
            let (r2, mut walls_2, mut doors_2, dir_2) = rooms_to_generate.remove(i);
            if i == 0 {
                doors_1.insert(WallDirection::Left);
            }
            if j + 1 == rooms_to_generate.len() + 2 {
                doors_2.insert(WallDirection::Bottom);
            }
            match dir_1 {
                WallDirection::Right if r1.area() <= r2.area() => {
                    walls_1.remove(&WallDirection::Right);
                    doors_2.insert(WallDirection::Left);
                }
                WallDirection::Right if r1.area() > r2.area() => {
                    doors_1.insert(WallDirection::Right);
                    walls_2.remove(&WallDirection::Left);
                }
                WallDirection::Bottom if r1.area() <= r2.area() => {
                    walls_1.remove(&WallDirection::Bottom);
                    doors_2.insert(WallDirection::Top);
                }
                WallDirection::Bottom if r1.area() > r2.area() => {
                    doors_1.insert(WallDirection::Bottom);
                    walls_2.remove(&WallDirection::Top);
                }
                _ => unreachable!(),
            }
            rooms_to_generate.insert(i, (r2, walls_2, doors_2, dir_2));
            rooms_to_generate.insert(i, (r1, walls_1, doors_1, dir_1));
        }
        let generated_rooms = rooms_to_generate
            .into_iter()
            .flat_map(|(rect, walls, doors, _)| {
                RoomGenerator::new(self.sprite_id, walls, doors).try_generate(&rect, state, cmds)
            })
            .collect_vec();
        generated_rooms.first().cloned()
    }
}

#[derive(Clone, Debug)]
pub struct RoomGenerator {
    pub sprite_id: &'static str,
    pub walls: HashSet<WallDirection>,
    pub doors: HashSet<WallDirection>,
}

impl RoomGenerator {
    pub fn new(
        sprite_id: &'static str,
        walls: HashSet<WallDirection>,
        doors: HashSet<WallDirection>,
    ) -> Self {
        Self {
            sprite_id,
            walls,
            doors,
        }
    }
}

impl EnvGenerator for RoomGenerator {
    fn try_generate(
        self,
        available_space: &Rect,
        _state: &State,
        cmds: &mut StateCommands,
    ) -> Option<EntityRef> {
        // println!("generating room {:?} for {:?}", self, available_space);
        let top_left = available_space.min;
        let room_size = available_space.w().min(available_space.h());
        let building = BuildingBundle::create(
            Transform::default()
                .translated((top_left.0 + room_size / 2., top_left.1 + room_size / 2.)),
            room_size,
            room_size,
            self.walls,
            self.doors,
            self.sprite_id,
            cmds,
        );
        Some(*building.primary_entity())
    }
}
