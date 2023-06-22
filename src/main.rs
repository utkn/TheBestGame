#![allow(dead_code)]

use std::collections::HashMap;

use itertools::Itertools;
use notan::{
    draw::{CreateDraw, DrawImages, DrawShapes, DrawTransform},
    egui::EguiPluginSugar,
    prelude::{AssetList, Assets},
};

use ai::*;
use camera::{map_to_screen_cords, map_to_world_cords};
use controller::{ControlSystem, UserInputDriver};
use effects::EffectSystem;
use game_entities::*;
use item::*;
use needs::*;
use physics::*;
use prelude::*;
use sprite::Sprite;
use vehicle::*;

mod ai;
mod camera;
mod controller;
mod effects;
mod game_entities;
mod item;
mod needs;
mod physics;
mod prelude;
mod sprite;
mod ui;
mod vehicle;

type AssetMap = HashMap<String, notan::prelude::Asset<notan::prelude::Texture>>;

#[derive(notan::AppState)]
struct AppState {
    world: World,
    ui_state: ui::UiState,
    asset_map: AssetMap,
}

fn setup(app: &mut notan::prelude::App, assets: &mut Assets) -> AppState {
    app.backend.window().set_title("TheBestGame v0");
    app.backend.window().set_size(960, 720);
    // Load the assets into memory.
    let asset_paths = glob::glob("./assets/**/*.png")
        .unwrap()
        .into_iter()
        .flatten()
        .flat_map(|path| path.into_os_string().into_string())
        .collect_vec();
    let asset_map: AssetMap = asset_paths
        .into_iter()
        .map(|asset_path| {
            (
                assets
                    .load_asset::<notan::prelude::Texture>(&asset_path)
                    .unwrap(),
                asset_path,
            )
        })
        .map(|(asset, path)| (path, asset))
        .collect();
    // Create the world from an empty state.
    let mut world = prelude::World::from(prelude::State::default());
    // Control & movement
    world.register_system(MovementSystem);
    world.register_system(AnchorSystem);
    world.register_system(ControlSystem::<UserInputDriver>::default());
    world.register_system(LifetimeSystem);
    world.register_system(ApproachVelocitySystem);
    world.register_system(FaceMouseSystem);
    // Interactions
    world.register_system(InteractionAcceptorSystem(
        ConsensusStrategy::MaxPriority,
        ConsensusStrategy::MinPriority,
    ));
    world.register_system(ProximityInteractionSystem);
    world.register_system(HandInteractionSystem);
    world.register_system(UntargetedInteractionDelegateSystem);
    // Basic physics
    world.register_system(CollisionDetectionSystem);
    world.register_system(SeparateCollisionsSystem);
    world.register_system(InteractionSystem::<Hitbox>::default());
    // Item stuff
    world.register_system(StorageSystem);
    world.register_system(EquipmentSystem);
    world.register_system(ItemTransferSystem);
    world.register_system(ItemAnchorSystem);
    world.register_system(ItemPickupSystem);
    world.register_system(InteractionSystem::<Item>::default());
    world.register_system(InteractionSystem::<Storage>::default());
    world.register_system(InteractionSystem::<Equipment>::default());
    // Needs
    world.register_system(NeedStateSystem::default());
    world.register_system(NeedMutatorSystem);
    // Projectiles
    world.register_system(InteractionSystem::<ProjectileGenerator>::default());
    world.register_system(ProjectileGenerationSystem);
    world.register_system(HitSystem);
    world.register_system(SuicideOnHitSystem);
    world.register_system(TimedEmitSystem::<GenerateProjectileReq>::default());
    world.register_system(ApplyOnHitSystem::<NeedMutator>::default());
    // Vehicle stuff
    world.register_system(VehicleSystem);
    world.register_system(InteractionSystem::<Vehicle>::default());
    // AI stuff
    world.register_system(VisionSystem);
    world.register_system(InteractionSystem::<VisionField>::default());
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
        asset_map,
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

fn draw_game(rnd: &mut notan::draw::Draw, state: &State, asset_map: &AssetMap) {
    state
        .select::<(Sprite, Transform)>()
        .flat_map(|(e, (sprite, trans))| {
            let assets_folder = std::path::PathBuf::from("assets");
            if let Some(asset_id) = sprite.get_file_path(assets_folder, &e, state) {
                Some((trans, asset_id.into_os_string().into_string().unwrap()))
            } else {
                None
            }
        })
        .flat_map(|(trans, asset_id)| {
            if let Some(asset) = asset_map.get(&asset_id) {
                Some((trans, asset))
            } else {
                None
            }
        })
        .for_each(|(trans, asset)| {
            asset.lock().map(|tx| {
                let (x, y) =
                    map_to_screen_cords(trans.x, trans.y, rnd.width(), rnd.height(), state);
                rnd.image(&tx)
                    .position(x - tx.width() / 2., y - tx.height() / 2.)
                    .rotate_degrees_from((x, y), -trans.deg);
            });
        });
}

fn draw_debug(rnd: &mut notan::draw::Draw, state: &State) {
    state
        .select::<(Transform, Hitbox)>()
        .for_each(|(e, (trans, hitbox))| {
            let is_being_interacted = state
                .select_one::<(InteractTarget<Storage>,)>(&e)
                .map(|(intr1,)| intr1.actors.len() > 0)
                .unwrap_or(false);
            let is_being_viewed = state
                .select_one::<(InteractTarget<VisionField>,)>(&e)
                .map(|(intr1,)| intr1.actors.len() > 0)
                .unwrap_or(false);
            let color = if is_being_interacted {
                notan::prelude::Color::GREEN
            } else if is_being_viewed {
                notan::prelude::Color::MAGENTA
            } else {
                notan::prelude::Color::RED
            };
            let (x, y) = map_to_screen_cords(trans.x, trans.y, rnd.width(), rnd.height(), state);
            let (dir_x, dir_y) = trans.dir_vec();
            rnd.circle(1.)
                .position(x, y)
                .fill_color(notan::prelude::Color::BLUE);
            rnd.line((x, y), (x + dir_x * 5., y + dir_y * 5.))
                .color(notan::prelude::Color::BLUE);
            match hitbox.1 {
                Shape::Circle(r) => {
                    rnd.circle(r).position(x, y).stroke(1.).stroke_color(color);
                }
                Shape::Rect(w, h) => {
                    rnd.rect((x - w / 2., y - h / 2.), (w, h))
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
    // Draw the game
    let mut game_rnd = gfx.create_draw();
    game_rnd.clear(notan::prelude::Color::new(0., 0., 0., 1.));
    draw_game(
        &mut game_rnd,
        app_state.world.get_state(),
        &mut app_state.asset_map,
    );
    draw_debug(&mut game_rnd, app_state.world.get_state());
    gfx.render(&game_rnd);
    // Draw the ui
    let egui_rnd = plugins.egui(|ctx| {
        app_state.world.update_with(|state, egui_cmds| {
            ui::draw_ui(ctx, state, egui_cmds, &mut app_state.ui_state);
        });
    });
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
