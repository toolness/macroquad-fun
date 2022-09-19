extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

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
mod drawing;
mod dynamic_platform;
mod entity;
mod flying_eye;
mod game_sprites;
mod ldtk;
mod level;
mod level_runtime;
mod math_util;
mod mushroom;
mod physics;
mod player;
mod running;
mod sprite_component;
mod sprite_renderer;
mod text;
mod time;
mod world;

#[macroquad::main("Fun")]
async fn main() {
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

    request_new_screen_size(config.screen_width, config.screen_height);
    next_frame().await;

    let mut level_runtime = new_game();

    loop {
        match level_runtime.advance_one_frame() {
            FrameResult::Ok => {}
            FrameResult::PlayerDied => {
                level_runtime = new_game();
            }
        }

        if is_key_released(KeyCode::Escape) {
            break;
        }

        next_frame().await;
    }
}

fn new_game() -> LevelRuntime {
    let (level_start, player_start) = world::world()
        .player_start()
        .expect("World must define a player start position");
    LevelRuntime::new(create_player(player_start), level_start)
}
