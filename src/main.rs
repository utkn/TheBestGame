#![allow(dead_code)]

use crate::core::*;
use camera::{map_to_screen_cords, map_to_world_cords};
use effects::EffectSystem;
use equipment::EquipmentSystem;
use game_entities::*;
use interaction::{
    HandInteractionSystem, Interactable, InteractionAcceptorSystem, InteractionDelegateSystem,
    InteractionSystem, ProximityInteractionSystem,
};
use item::{Item, ItemAnchorSystem, ItemPickupSystem, ItemTransferSystem};
use misc_systems::*;
use needs::*;
use notan::{
    draw::{CreateDraw, DrawShapes},
    egui::EguiPluginSugar,
};
use physics::*;
use projectile::{
    ApplyOnHitSystem, GenerateProjectileReq, ProjectileGenerationSystem, ProjectileGenerator,
    ProjectileHitSystem, SuicideOnHitSystem,
};
use storage::{Storage, StorageSystem};
use timed::{TimedEmitSystem, TimedRemoveSystem};
use ui::{draw_ui, UiState};
use vehicle::{Vehicle, VehicleSystem};

mod camera;
mod core;
mod effects;
mod entity_insights;
mod equipment;
mod game_entities;
mod interaction;
mod item;
mod misc_systems;
mod needs;
mod physics;
mod projectile;
mod storage;
mod timed;
mod ui;
mod vehicle;

#[derive(notan::AppState)]
struct AppState {
    world: World,
    ui_state: UiState,
}

fn setup(app: &mut notan::prelude::App) -> AppState {
    app.backend.window().set_title("TheBestGame v0");
    // Create the world from an empty state.
    let mut world = core::World::from(core::State::default());
    // Control & movement
    world.register_system(MovementSystem);
    world.register_system(AnchorSystem);
    world.register_system(ControlSystem);
    world.register_system(LifetimeSystem);
    world.register_system(ApproachVelocitySystem);
    world.register_system(FaceMouseSystem);
    // Basic physics
    world.register_system(CollisionDetectionSystem::default());
    world.register_system(SeparateCollisionsSystem);
    // Interactions
    world.register_system(InteractionAcceptorSystem);
    world.register_system(ProximityInteractionSystem);
    world.register_system(HandInteractionSystem);
    world.register_system(InteractionDelegateSystem);
    // Item stuff
    world.register_system(StorageSystem);
    world.register_system(EquipmentSystem);
    world.register_system(ItemTransferSystem);
    world.register_system(ItemAnchorSystem);
    world.register_system(ItemPickupSystem);
    world.register_system(InteractionSystem::<Item>::default());
    world.register_system(InteractionSystem::<Storage>::default());
    // Needs
    world.register_system(NeedStateSystem::default());
    world.register_system(NeedMutatorSystem);
    // Projectiles
    world.register_system(InteractionSystem::<ProjectileGenerator>::default());
    world.register_system(ProjectileGenerationSystem);
    world.register_system(ProjectileHitSystem);
    world.register_system(SuicideOnHitSystem);
    world.register_system(TimedEmitSystem::<GenerateProjectileReq>::default());
    world.register_system(ApplyOnHitSystem::<NeedMutator>::default());
    // Riding
    world.register_system(VehicleSystem::default());
    world.register_system(InteractionSystem::<Vehicle>::default());
    // Misc
    world.register_system(TimedRemoveSystem::<NeedMutator>::default());
    world.register_system(EffectSystem::<MaxSpeed>::default());
    world.register_system(EffectSystem::<Acceleration>::default());
    world.register_system(ExistenceDependencySystem);
    // Initialize the scene for debugging.
    world.update_with(|_, cmds| {
        create_player(Transform::at(0., 0.), cmds);
        create_chest(Transform::at(50., 50.), cmds);
        create_handgun(Transform::at(150., 150.), Name("gun"), cmds);
        create_handgun(Transform::at(150., 150.), Name("gun"), cmds);
        create_machinegun(Transform::at(200., 200.), Name("machine gun"), cmds);
        create_shoes(Transform::at(180., 180.), Name("shoes"), cmds);
        create_vehicle(Transform::at(500., 500.), cmds);
    });
    AppState {
        world,
        ui_state: Default::default(),
    }
}

fn update(app: &mut notan::prelude::App, app_state: &mut AppState) {
    let dt = app.timer.delta_f32();
    let mut control_map = ControlMap::from_app_state(&app);
    control_map.mouse_pos = map_to_world_cords(
        control_map.mouse_pos.0,
        control_map.mouse_pos.1,
        app.window().width() as f32,
        app.window().height() as f32,
        app_state.world.get_state(),
    );
    app_state
        .world
        .update_with_systems(UpdateContext { dt, control_map });
}

fn draw_game(rnd: &mut notan::draw::Draw, state: &State) {
    state
        .select::<(Transform, Hitbox)>()
        .for_each(|(e, (trans, hitbox))| {
            let is_being_interacted = state
                .select_one::<(Interactable<Storage>,)>(&e)
                .map(|(interactable,)| interactable.actors.len() > 0)
                .unwrap_or(false);
            let color = if is_being_interacted {
                notan::prelude::Color::GREEN
            } else {
                notan::prelude::Color::RED
            };
            let (x, y) = map_to_screen_cords(trans.x, trans.y, rnd.width(), rnd.height(), state);
            match hitbox.1 {
                Shape::Circle(r) => {
                    rnd.circle(r).position(x, y).stroke(1.).stroke_color(color);
                }
                Shape::Rect(w, h) => {
                    rnd.rect((x, y), (w, h)).stroke(1.).stroke_color(color);
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
