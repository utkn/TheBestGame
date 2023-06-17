use crate::{
    activation::ActivatedEvt,
    core::*,
    physics::{Hitbox, HitboxType, Shape},
};

#[derive(Clone, Copy, Debug)]
pub struct ProjectileDefn {
    pub lifetime: f32,
    pub speed: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ProjectileGenerator(pub ProjectileDefn);

#[derive(Clone, Copy, Debug)]
pub struct ProjectileGenerationSystem;

impl System for ProjectileGenerationSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state.read_events::<ActivatedEvt>().for_each(|evt| {
            if let Some((p_gen, pos, rot)) =
                state.select_one::<(ProjectileGenerator, Position, Rotation)>(&evt.0)
            {
                let vel = notan::math::Vec2::from_angle(rot.deg.to_radians()) * p_gen.0.speed;
                let vel = Velocity {
                    x: vel.x,
                    y: -vel.y,
                };
                let hitbox = Hitbox(HitboxType::Ghost, Shape::Circle(5.));
                cmds.create_from((*pos, vel, Lifetime(p_gen.0.lifetime), hitbox));
            }
        });
    }
}
