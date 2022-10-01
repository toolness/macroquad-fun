extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::env::args;

use config::load_config;
use game_sprites::load_game_sprites;
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
mod flying_eye;
mod font;
mod game_sprites;
mod ldtk;
mod level;
mod level_runtime;
mod math_util;
mod moving_platform;
mod mushroom;
mod physics;
mod player;
mod route;
mod running;
mod sprite_component;
mod sprite_renderer;
mod text;
mod time;
mod world;
mod xy_range_iterator;
mod z_index;

const DEFAULT_START_POSITION: &str = "default";

#[macroquad::main("Fun")]
async fn main() {
    let args: Vec<String> = args().collect();
    let start_position = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or(&DEFAULT_START_POSITION);

    load_config("media/config.json")
        .await
        .expect("load_config() must succeed");
    load_game_sprites()
        .await
        .expect("load_game_sprites() must succeed");
    load_world("media/world.ldtk")
        .await
        .expect("load_world() must succeed");

    let config = config::config();
    let mut level_runtime = new_game(start_position);

    request_new_screen_size(config.screen_width, config.screen_height);
    next_frame().await;

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
    LevelRuntime::new(create_player(player_start), level_start)
}
