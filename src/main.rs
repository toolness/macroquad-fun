extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use config::load_config;
use game_assets::load_game_assets;
use level_runtime::{FrameResult, LevelRuntime};
use macroquad::prelude::*;
use player::create_player;
use world::load_world;

mod animator;
mod aseprite;
mod attachment;
mod camera;
mod collision;
mod config;
mod crate_entity;
mod drawing;
mod dynamic_collider;
mod entity;
mod floor_switch;
mod flying_eye;
mod font;
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

const DEFAULT_START_POSITION: &str = "default";

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

#[derive(Default)]
#[cfg_attr(not(target_arch = "wasm32"), derive(clap::Parser))]
struct Cli {
    start_position: Option<String>,
}

impl Cli {
    pub fn get_for_platform() -> Self {
        #[cfg(target_arch = "wasm32")]
        return Cli::default();

        #[cfg(not(target_arch = "wasm32"))]
        <Cli as clap::Parser>::parse()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args = Cli::get_for_platform();
    let start_position = args
        .start_position
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or(&DEFAULT_START_POSITION);

    load_config(CONFIG_PATH)
        .await
        .expect("load_config() must succeed");
    load_game_assets()
        .await
        .expect("load_game_sprites() must succeed");
    load_world("media/world.ldtk")
        .await
        .expect("load_world() must succeed");

    let mut level_runtime = new_game(start_position);

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

    loop {
        match level_runtime.advance_one_frame() {
            FrameResult::Ok => {}
            FrameResult::PlayerDied => {
                level_runtime = new_game(start_position);
            }
        }

        if is_key_released(KeyCode::Escape) {
            break;
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
