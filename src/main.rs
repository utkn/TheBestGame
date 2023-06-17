#![allow(dead_code)]

use crate::core::primitive_components::*;
use crate::core::*;
use equipment::EquipmentSystem;
use game_entities::{create_chest, create_item, create_player};
use interaction::{
    Interactable, InteractionSystem, ProximityInteractionSystem, ProximityInteractor,
};
use item::{ItemPickupSystem, ItemTransferSystem};
use notan::{
    draw::{CreateDraw, CreateFont, DrawShapes},
    egui::EguiPluginSugar,
};
use physics::{CollisionDetectionSystem, Hitbox, SeparateCollisionsSystem, Shape};
use storage::StorageSystem;
use ui::{draw_ui, UiState};

mod core;
mod equipment;
mod game_entities;
mod interaction;
mod item;
mod physics;
mod storage;
mod ui;

#[derive(Clone, Copy, Debug, Default)]
pub struct MovementSystem;

impl System for MovementSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(Position, Velocity)>()
            .for_each(|(e, (pos, vel))| {
                let mut new_pos = *pos;
                new_pos.x += vel.x * ctx.dt;
                new_pos.y += vel.y * ctx.dt;
                cmds.set_component(&e, new_pos);
            });
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlSystem;

impl System for ControlSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(Velocity, TargetVelocity, Controller)>()
            .for_each(|(e, (_, _, controller))| {
                let new_target_vel_x = if ctx.control_map.left_pressed {
                    -1.
                } else if ctx.control_map.right_pressed {
                    1.
                } else {
                    0.
                } * controller.max_speed;
                let new_target_vel_y = if ctx.control_map.up_pressed {
                    -1.
                } else if ctx.control_map.down_pressed {
                    1.
                } else {
                    0.
                } * controller.max_speed;
                cmds.set_component(
                    &e,
                    TargetVelocity {
                        x: new_target_vel_x,
                        y: new_target_vel_y,
                    },
                )
            })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ApproachVelocitySystem;

impl System for ApproachVelocitySystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(Velocity, TargetVelocity, Acceleration)>()
            .for_each(|(e, (vel, target_vel, acc))| {
                let vel = notan::math::vec2(vel.x, vel.y);
                let target_vel = notan::math::vec2(target_vel.x, target_vel.y);
                let new_vel = vel + acc.0 * ctx.dt * (target_vel - vel).normalize_or_zero();
                cmds.set_component(
                    &e,
                    Velocity {
                        x: new_vel.x,
                        y: new_vel.y,
                    },
                )
            })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AnchorSystem;

impl System for AnchorSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(AnchorPosition,)>()
            .for_each(|(child_entity, (anchor,))| {
                if !state.is_valid(&anchor.0) {
                    cmds.remove_entity(&child_entity);
                } else if let Some((anchored_pos,)) = state.select_one::<(Position,)>(&anchor.0) {
                    let new_pos = anchored_pos.translated(anchor.1);
                    cmds.set_component(&child_entity, new_pos);
                }
            })
    }
}

#[derive(notan::AppState)]
struct AppState {
    world: World,
    font: notan::draw::Font,
    ui_state: UiState,
}

fn setup(gfx: &mut notan::prelude::Graphics) -> AppState {
    let font = gfx
        .create_font(include_bytes!("assets/Ubuntu-B.ttf"))
        .unwrap();
    let mut world = core::World::from(core::State::default());
    world.register_system(MovementSystem);
    world.register_system(ControlSystem::default());
    world.register_system(ApproachVelocitySystem);
    world.register_system(CollisionDetectionSystem::default());
    world.register_system(SeparateCollisionsSystem);
    world.register_system(InteractionSystem::default());
    world.register_system(ProximityInteractionSystem);
    world.register_system(StorageSystem);
    world.register_system(EquipmentSystem);
    world.register_system(ItemTransferSystem);
    world.register_system(ItemPickupSystem);
    world.register_system(AnchorSystem);
    world.update_with(|_, cmds| {
        create_player(cmds, Position { x: 0., y: 0. });
        create_chest(cmds, Position { x: 50., y: 50. });
        create_item(cmds, Position { x: 150., y: 150. }, Name("thing"));
        create_item(cmds, Position { x: 150., y: 150. }, Name("other thing"));
    });
    AppState {
        world,
        font,
        ui_state: Default::default(),
    }
}

fn update(app: &mut notan::prelude::App, app_state: &mut AppState) {
    let dt = app.timer.delta_f32();
    let control_map = ControlMap::from_app_state(&app);
    app_state
        .world
        .update_with_systems(UpdateContext { dt, control_map });
}

fn draw_game(rnd: &mut notan::draw::Draw, state: &State) {
    state
        .select::<(Position, Hitbox)>()
        .for_each(|(e, (pos, hitbox))| {
            let is_activated = state
                .select_one::<(Interactable,)>(&e)
                .map(|(interactable,)| interactable.actors.len() > 0)
                .unwrap_or(false);
            let is_activator = state
                .select_one::<(ProximityInteractor,)>(&e)
                .map(|(pi,)| pi.target.is_some())
                .unwrap_or(false);
            let color = if is_activated {
                notan::prelude::Color::GREEN
            } else if is_activator {
                notan::prelude::Color::BLUE
            } else {
                notan::prelude::Color::RED
            };
            match hitbox.1 {
                Shape::Circle(r) => {
                    rnd.circle(r)
                        .position(pos.x, pos.y)
                        .stroke(1.)
                        .stroke_color(color);
                }
                Shape::Rect(w, h) => {
                    rnd.rect((pos.x, pos.y), (w, h))
                        .stroke(1.)
                        .stroke_color(color);
                }
            };
        });
}

fn draw(
    _app: &mut notan::prelude::App,
    gfx: &mut notan::prelude::Graphics,
    plugins: &mut notan::prelude::Plugins,
    app_state: &mut AppState,
) {
    let egui_rnd = plugins.egui(|ctx| {
        app_state.world.update_with(|state, egui_cmds| {
            draw_ui(ctx, state, egui_cmds, &mut app_state.ui_state);
        });
    });
    // Draw the game
    let mut game_rnd = gfx.create_draw();
    game_rnd.clear(notan::prelude::Color::new(0., 0., 0., 1.));
    draw_game(&mut game_rnd, app_state.world.get_state());
    gfx.render(&game_rnd);
    // Draw the ui
    gfx.render(&egui_rnd);
}

#[notan::notan_main]
fn main() -> Result<(), String> {
    notan::init_with(setup)
        .update(update)
        .add_config(notan::egui::EguiConfig)
        .add_config(notan::draw::DrawConfig)
        .draw(draw)
        .build()
}
