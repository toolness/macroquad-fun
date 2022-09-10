extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use camera::center_camera;
use config::load_config;
use drawing::draw_rect_lines;
use game_sprites::load_game_sprites;
use level_runtime::LevelRuntime;
use macroquad::prelude::*;
use player::Player;
use text::draw_level_text;
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
    let mut level_runtime = LevelRuntime::new(level_start);

    request_new_screen_size(config.screen_width, config.screen_height);
    next_frame().await;

    let mut player = Player::new(player_start);
    let mut last_frame_time = get_time();
    let mut debug_mode = false;

    loop {
        // Keep track of time.
        let now = get_time();
        let absolute_frame_number = (now * 1000.0 / config.ms_per_animation_frame) as u32;
        let time_since_last_frame = now - last_frame_time;

        last_frame_time = now;

        // If the player isn't mostly inside the current level, change levels.
        if let Some(new_level) = player.maybe_switch_levels(&level_runtime.level) {
            level_runtime = LevelRuntime::new(new_level);
        }

        let level = level_runtime.level;

        // Position the camera.
        let camera_rect = center_camera(&player, &level);

        // Draw environment.
        clear_background(GRAY);
        level.draw(&camera_rect);

        // Process input/physics.
        player.process_input_and_physics(&level, time_since_last_frame);

        // Draw entities.

        for flying_eye in level_runtime.flying_eyes.iter() {
            flying_eye.entity().draw(absolute_frame_number);
        }

        player.entity().draw(absolute_frame_number);

        draw_level_text(&player, &level, &camera_rect);

        // Process miscellaneous system input.

        if is_key_released(KeyCode::Escape) {
            break;
        } else if is_key_pressed(KeyCode::GraveAccent) {
            debug_mode = !debug_mode;
        }

        if debug_mode {
            draw_debug_layer(&player, &level_runtime, &camera_rect);
        }

        // Wait for the next frame.

        next_frame().await;
    }
}

pub fn draw_debug_layer(player: &Player, level_runtime: &LevelRuntime, camera_rect: &Rect) {
    let level = level_runtime.level;
    player.entity().draw_debug_rects();
    for collider in level.iter_colliders(&level.pixel_bounds()) {
        collider.draw_debug_rect(PURPLE);
    }
    draw_rect_lines(
        &level.get_bounding_cell_rect(&player.entity().bbox()),
        1.,
        WHITE,
    );
    for flying_eye in level_runtime.flying_eyes.iter() {
        flying_eye.entity().draw_debug_rects();
    }
    let text = format!("fps: {}", get_fps());
    draw_text(&text, camera_rect.x + 32., camera_rect.y + 32., 32.0, WHITE);
}
