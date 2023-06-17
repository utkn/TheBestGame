#![allow(dead_code)]

use crate::core::*;
use activation::ActivationSystem;
use equipment::EquipmentSystem;
use game_entities::{create_chest, create_item, create_player};
use interaction::*;
use item::{EquippedItemAnchorSystem, ItemPickupSystem, ItemTransferSystem};
use misc_systems::*;
use needs::*;
use notan::{
    draw::{CreateDraw, DrawShapes},
    egui::EguiPluginSugar,
};
use physics::*;
use projectile::ProjectileGenerationSystem;
use storage::StorageSystem;
use ui::{draw_ui, UiState};

mod activation;
mod core;
mod equipment;
mod game_entities;
mod interaction;
mod item;
mod misc_systems;
mod needs;
mod physics;
mod projectile;
mod storage;
mod ui;

#[derive(notan::AppState)]
struct AppState {
    world: World,
    ui_state: UiState,
}

fn setup(app: &mut notan::prelude::App) -> AppState {
    app.backend.window().set_title("TheBestGame v0");
    // Create the world from an empty state.
    let mut world = core::World::from(core::State::default());
    // Register the systems.
    world.register_system(MovementSystem);
    world.register_system(ControlSystem::default());
    world.register_system(LifetimeSystem);
    world.register_system(ApproachVelocitySystem);
    world.register_system(FaceMouseSystem);
    world.register_system(CollisionDetectionSystem::default());
    world.register_system(SeparateCollisionsSystem);
    world.register_system(InteractionSystem::default());
    world.register_system(ProximityInteractionSystem);
    world.register_system(HandInteractionSystem);
    world.register_system(StorageSystem);
    world.register_system(EquipmentSystem);
    world.register_system(ItemTransferSystem);
    world.register_system(ItemPickupSystem);
    world.register_system(EquippedItemAnchorSystem);
    world.register_system(AnchorSystem);
    world.register_system(NeedsSystem::default());
    world.register_system(ActivationSystem);
    world.register_system(ProjectileGenerationSystem);
    // Initialize the scene for debugging.
    world.update_with(|_, cmds| {
        create_player(cmds, Position { x: 0., y: 0. });
        create_chest(cmds, Position { x: 50., y: 50. });
        create_item(cmds, Position { x: 150., y: 150. }, Name("thing"));
        create_item(cmds, Position { x: 150., y: 150. }, Name("other thing"));
    });
    AppState {
        world,
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
