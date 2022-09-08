extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use camera::calculate_camera_rect;
use config::load_config;
use drawing::draw_rect_lines;
use flying_eye::FlyingEye;
use game_sprites::load_game_sprites;
use level::World;
use macroquad::prelude::*;
use player::Player;

mod aseprite;
mod camera;
mod collision;
mod config;
mod drawing;
mod flying_eye;
mod game_sprites;
mod ldtk;
mod level;
mod player;
mod running;
mod sprite;

#[macroquad::main("Fun")]
async fn main() {
    let config = load_config("media/config.json").await.unwrap();
    let sprite_scale = config.sprite_scale;
    let world = World::load("media/world.ldtk", sprite_scale).await.unwrap();
    let (mut level, player_start) = world
        .player_start()
        .expect("World must define a player start position");
    let sprites = load_game_sprites(sprite_scale).await.unwrap();

    request_new_screen_size(config.screen_width, config.screen_height);
    next_frame().await;

    let mut player = Player::new(player_start, &sprites);
    let mut last_frame_time = get_time();
    let mut debug_mode = false;

    loop {
        // Keep track of time.
        let now = get_time();
        let absolute_frame_number = (now * 1000.0 / config.ms_per_animation_frame) as u32;
        let time_since_last_frame = now - last_frame_time;

        last_frame_time = now;

        // If the player isn't mostly inside the current level, change levels.
        if let Some(new_level) = player.maybe_switch_levels(&level, &world) {
            level = new_level;
        }

        // Position the camera.
        let camera_rect: Rect;
        {
            let bbox = player.bbox();
            let bbox_center = Vec2::new(bbox.x + bbox.w / 2., bbox.y + bbox.h / 2.);
            camera_rect = calculate_camera_rect(&config, &bbox_center, &level.pixel_bounds());
            set_camera(&Camera2D::from_display_rect(camera_rect));
        }

        // Draw environment.

        clear_background(GRAY);
        level.draw(&camera_rect);

        // Process input/physics.
        player.process_input_and_physics(&config, &level, time_since_last_frame);

        // Draw NPCs.

        for flying_eye_rect in level.iter_flying_eyes() {
            let flying_eye = FlyingEye::new(flying_eye_rect, &sprites);
            flying_eye.draw(&sprites, absolute_frame_number);
            if debug_mode {
                flying_eye.draw_debug_rects(&sprites);
            }
        }

        // Draw player.
        player.draw(&sprites, absolute_frame_number);

        // Draw level text.
        if let Some(text) = level.get_text(&player.bbox()) {
            let mut y = camera_rect.y + 128.;
            for line in text {
                draw_text(line, camera_rect.x + 32., y, 32.0, WHITE);
                y += 36.;
            }
        }

        // Process miscellaneous system input.

        if is_key_released(KeyCode::Escape) {
            break;
        } else if is_key_pressed(KeyCode::GraveAccent) {
            debug_mode = !debug_mode;
        }
        if debug_mode {
            player.draw_debug_rects(&sprites);
            for collider in level.iter_colliders(&level.pixel_bounds()) {
                collider.draw_debug_rect(PURPLE);
            }
            draw_rect_lines(&level.get_bounding_cell_rect(&player.bbox()), 1., WHITE);
            let text = format!("fps: {}", get_fps());
            draw_text(&text, camera_rect.x + 32., camera_rect.y + 32., 32.0, WHITE);
        }

        // Wait for the next frame.

        next_frame().await;
    }
}
