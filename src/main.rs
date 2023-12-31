#![allow(dead_code)]

use std::{collections::HashMap, path::PathBuf};

use itertools::Itertools;
use notan::{
    draw::{self, CreateDraw, DrawImages, DrawShapes, DrawTransform},
    egui::EguiPluginSugar,
    prelude::{Asset, Assets, Texture},
};

use camera::*;
use item::*;
use physics::*;
use prelude::*;
use sprite::*;
use world_gen::*;

mod ai;
mod building;
mod camera;
mod character;
mod chunks;
mod controller;
mod effects;
mod item;
mod needs;
mod physics;
mod prelude;
mod sprite;
mod ui;
mod vehicle;
mod world_gen;

/// Maps an asset path to the loaded texture.
type AssetMap = HashMap<PathBuf, Asset<Texture>>;

#[derive(notan::AppState)]
struct AppState {
    world: SystemManager<State>,
    ui_state: ui::UiState,
    asset_map: AssetMap,
    sprite_representor: SpriteRepresentor,
}

fn setup(app: &mut notan::prelude::App, assets: &mut Assets) -> AppState {
    app.backend.window().set_title("TheBestGame v0");
    app.backend.window().set_size(960, 720);
    // Load the assets into the memory.
    let asset_paths = glob::glob("./assets/**/*.png")
        .unwrap()
        .into_iter()
        .flatten()
        .collect_vec();
    let asset_map: AssetMap = asset_paths
        .into_iter()
        .map(|asset_path| {
            // Load the texture from the asset path.
            let asset_path_str = asset_path.as_path().to_str().unwrap();
            let tx = assets
                .load_asset::<notan::prelude::Texture>(asset_path_str)
                .unwrap();
            (asset_path, tx)
        })
        .collect();
    // Generate a debugging world.
    let mut world = WorldGenerator::generate(WorldTemplate::new([
        (Transform::at(-40., -40.), PLAYER_TEMPLATE),
        // (Transform::at(50., 50.), CHEST_TEMPLATE),
        (Transform::at(500., 500.), BASIC_CAR_TEMPLATE),
        // (Transform::at(10., 10.), HAND_GUN_TEMPLATE),
        (Transform::at(10., 10.), MACHINE_GUN_TEMPLATE),
        (Transform::at(10., 10.), SIMPLE_BACKPACK_TEMPLATE),
        (Transform::at(10., 10.), RUNNING_SHOES_TEMPLATE),
        // (Transform::at(-50., -50.), BANDIT_TEMPLATE),
    ]));
    let house_size = 4096.;
    world.update_with(|state, cmds| {
        HouseGenerator::new("derelict_house").try_generate(
            &Rect::new((0., 0.), (house_size, house_size)),
            state,
            cmds,
        );
        // HouseGenerator::new("derelict_house").try_generate(
        //     &Rect::new((-600., -600.), (house_size, house_size)),
        //     state,
        //     cmds,
        // );
    });
    AppState {
        world,
        asset_map,
        ui_state: Default::default(),
        sprite_representor: Default::default(),
    }
}

fn update(app: &mut notan::prelude::App, app_state: &mut AppState) {
    // let dt = app.timer.delta_f32();
    let dt = 1. / 60.;
    let mut control_map = ControlMap::from_app_state(&app);
    // Move the mouse into the world coordinates.
    control_map.mouse_pos = map_to_world_cords(
        control_map.mouse_pos.0,
        control_map.mouse_pos.1,
        app.window().width() as f32,
        app.window().height() as f32,
        app_state.world.get_state(),
    );
    // Update the world with the registered systems.
    let world = &mut app_state.world;
    world.update_with_systems(UpdateContext { dt, control_map });
}

fn draw_sprite(
    rnd: &mut draw::Draw,
    x: f32,
    y: f32,
    deg: f32,
    tiling_config: TilingConfig,
    tx: &Texture,
) {
    let repeat_x = tiling_config.repeat_x;
    let repeat_y = tiling_config.repeat_y;
    let tx_x = x - (tx.width() * (repeat_x as f32)) / 2.;
    let tx_y = y - (tx.height() * (repeat_y as f32)) / 2.;
    for row in 0..repeat_y {
        for col in 0..repeat_x {
            let tx_x = tx_x + tx.width() * col as f32;
            let tx_y = tx_y + tx.height() * row as f32;
            rnd.image(tx.as_ref())
                .position(tx_x, tx_y)
                .rotate_degrees_from((x, y), -deg);
        }
    }
}

fn draw_game(rnd: &mut draw::Draw, app_state: &mut AppState) {
    let game_state = app_state.world.get_state();
    let draw_bounds = game_state
        .select::<(Transform, CameraFollow)>()
        .next()
        .map(|(_, (trans, camera))| {
            (
                (trans.x - camera.w / 2., trans.y - camera.h / 2.),
                (trans.x + camera.w / 2., trans.y + camera.h / 2.),
            )
        })
        .unwrap_or_default();
    game_state
        .select::<(Transform, Sprite)>()
        .filter(|(_, (trans, _))| {
            trans.x.clamp(draw_bounds.0 .0, draw_bounds.1 .0) == trans.x
                && trans.y.clamp(draw_bounds.0 .1, draw_bounds.1 .1) == trans.y
        })
        .flat_map(|(sprite_entity, (trans, sprite))| {
            app_state
                .sprite_representor
                .get_representations(&sprite_entity, game_state)
                .next()
                .and_then(|sprite_frames| sprite_frames.get_corresponding_frame(sprite))
                .and_then(|path| app_state.asset_map.get(path))
                .map(|tx| (trans, sprite, tx))
        })
        .sorted_by_key(|(_, sprite, _)| sprite.z_index)
        .for_each(|(trans, sprite, tx)| {
            if let Some(tx) = tx.lock() {
                let (x, y) =
                    map_to_screen_cords(trans.x, trans.y, rnd.width(), rnd.height(), game_state);
                draw_sprite(rnd, x, y, trans.deg, sprite.tiling_config, tx.as_ref());
            }
        });
}

fn draw_debug(rnd: &mut draw::Draw, state: &impl StateReader) {
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
                notan::prelude::Color::YELLOW
            } else {
                notan::prelude::Color::RED
            };
            let (x, y) = map_to_screen_cords(trans.x, trans.y, rnd.width(), rnd.height(), state);
            rnd.circle(2.)
                .position(x, y)
                .rotate_degrees_from((x, y), -trans.deg)
                .fill_color(notan::prelude::Color::BLUE);
            match hitbox.1 {
                Shape::Circle { r } => {
                    rnd.circle(r)
                        .position(x, y)
                        .stroke(1.)
                        .rotate_degrees_from((x, y), -trans.deg)
                        .stroke_color(color);
                }
                Shape::Rect { w, h } => {
                    rnd.rect((x - w / 2., y - h / 2.), (w, h))
                        .rotate_degrees_from((x, y), -trans.deg)
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
    game_rnd.clear(notan::prelude::Color::GRAY);
    draw_game(&mut game_rnd, app_state);
    // draw_debug(&mut game_rnd, app_state.world.get_state());
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
