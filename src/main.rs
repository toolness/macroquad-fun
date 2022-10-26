extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use cli::Cli;
use config::load_config;
use debug_mode::DebugMode;
use fps::FpsCounter;
use game_assets::load_game_assets;
use input::InputState;
use level_runtime::{FrameResult, LevelRuntime};
use macroquad::prelude::*;
use player::create_player;
use time::GameTime;
use world::load_world;

mod animator;
mod aseprite;
mod attachment;
mod camera;
mod cli;
mod collision;
mod config;
mod crate_entity;
mod debug_mode;
mod drawing;
mod dynamic_collider;
mod entity;
mod floor_switch;
mod flying_eye;
mod font;
mod fps;
mod game_assets;
mod input;
mod ldtk;
mod level;
mod level_runtime;
mod math_util;
mod moving_platform;
mod mushroom;
mod physics;
mod player;
mod push;
mod route;
mod running;
mod sprite_component;
mod sprite_renderer;
mod switch;
mod text;
mod time;
mod world;
mod xy_range_iterator;
mod z_index;

const CONFIG_PATH: &str = "media/config.json";

fn window_conf() -> Conf {
    #[cfg(target_arch = "wasm32")]
    return Default::default();

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Unfortunately this function isn't async, so we can't use Macroquad's WASM-compatible
        // async filesystem functions here. So all this code is specific to native builds.
        //
        // MacOS in particular won't let us change window dimensions after launching, so we'll
        // need to parse the config synchronously here and pass the correct dimensions to it.
        let config = config::parse_config(&std::fs::read_to_string(CONFIG_PATH).unwrap())
            .expect("parse_config() must succeed");

        Conf {
            window_title: "Macroquad Fun".to_owned(),
            window_width: config.screen_width as i32,
            window_height: config.screen_height as i32,
            window_resizable: false,
            ..Default::default()
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args = Cli::get_for_platform();

    load_config(CONFIG_PATH)
        .await
        .expect("load_config() must succeed");
    load_game_assets()
        .await
        .expect("load_game_sprites() must succeed");
    load_world("media/world.ldtk")
        .await
        .expect("load_world() must succeed");

    let mut level_runtime = new_game(&args.start_position);

    #[cfg(target_arch = "wasm32")]
    {
        // I'm unclear on whether this is actually needed on wasm32, but
        // just in case...
        //
        // (Note that native builds will have already set the screen size
        // properly via our window_conf function.)
        let config = config::config();
        request_new_screen_size(config.screen_width, config.screen_height);
        next_frame().await;
    }

    let mut time = GameTime::new(get_time());
    let mut enable_debug_mode = false;
    let mut opt_debug_mode: Option<DebugMode> = None;
    let mut fps = FpsCounter::default();

    loop {
        time.update(get_time());
        fps.update(time.now);

        match level_runtime.advance_one_frame(&time, &InputState::from_macroquad()) {
            FrameResult::Ok => {}
            FrameResult::PlayerDied => {
                level_runtime = new_game(&args.start_position);
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        if is_key_released(KeyCode::Escape) {
            break;
        }

        if is_key_pressed(KeyCode::G) {
            enable_debug_mode = !enable_debug_mode;
        }

        if enable_debug_mode {
            let debug_mode = opt_debug_mode.get_or_insert_with(|| DebugMode::default());
            debug_mode
                .update(&level_runtime, &fps)
                .expect("Generating debug text should work!");
            debug_mode.draw(&level_runtime);
        }

        next_frame().await;
    }
}

fn new_game(start_position: &str) -> LevelRuntime {
    let (level_start, player_start) = world::world().player_start(start_position).expect(&format!(
        "World does not define a PlayerStart entity called '{}'!",
        start_position
    ));
    LevelRuntime::new(create_player(player_start, "PLAYER"), level_start)
}
