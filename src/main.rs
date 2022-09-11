extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use config::load_config;
use game_sprites::load_game_sprites;
use level_runtime::LevelRuntime;
use macroquad::prelude::*;
use player::Player;
use world::load_world;

mod aseprite;
mod camera;
mod collision;
mod config;
mod drawing;
mod flying_eye;
mod game_sprites;
mod ldtk;
mod level;
mod level_runtime;
mod player;
mod running;
mod sprite;
mod sprite_entity;
mod text;
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
    let (level_start, player_start) = world::world()
        .player_start()
        .expect("World must define a player start position");
    let mut level_runtime = LevelRuntime::new(Player::new(player_start), level_start);

    request_new_screen_size(config.screen_width, config.screen_height);
    next_frame().await;

    level_runtime.run().await;
}
