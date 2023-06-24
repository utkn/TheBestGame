use crate::prelude::*;

use super::HitEvt;

pub trait ProjectileInsights<'a> {
    fn new_hitters_of(&self, target: &EntityRef) -> Vec<(&'a EntityRef, &'a (f32, f32))>;
}

impl<'a> ProjectileInsights<'a> for StateInsights<'a> {
    fn new_hitters_of(&self, target: &EntityRef) -> Vec<(&'a EntityRef, &'a (f32, f32))> {
        self.0
            .read_events::<HitEvt>()
            .filter(|evt| &evt.target == target)
            .map(|evt| (&evt.hitter, &evt.hit_velocity))
            .collect()
    }
}
