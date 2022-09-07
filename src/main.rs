extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use camera::calculate_camera_rect;
use config::load_config;
use drawing::draw_rect_lines;
use game_sprites::load_game_sprites;
use level::World;
use macroquad::prelude::*;
use running::RunManager;
use sprite::{Sprite, SpriteDrawParams};

use crate::collision::{process_collision, Actor};

mod aseprite;
mod camera;
mod collision;
mod config;
mod drawing;
mod game_sprites;
mod ldtk;
mod level;
mod running;
mod sprite;

pub struct Player {
    pub pos: Vec2,
    pub is_in_air: bool,
    pub velocity: Vec2,
    pub is_facing_left: bool,
    pub relative_bbox: Rect,
}

impl Player {
    pub fn new(start_rect: Rect, relative_bbox: Rect) -> Self {
        Player {
            pos: Vec2::new(
                start_rect.left() - relative_bbox.x,
                start_rect.bottom() - relative_bbox.bottom(),
            ),
            relative_bbox,
            is_in_air: false,
            velocity: Vec2::new(0., 0.),
            is_facing_left: false,
        }
    }

    pub fn bbox(&self) -> Rect {
        self.relative_bbox.offset(self.pos)
    }
}

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

    let mut player = Player::new(player_start, sprites.huntress.idle_bbox);
    let mut last_frame_time = get_time();
    let mut debug_mode = false;
    let mut run_manager = RunManager::new();

    loop {
        // Keep track of time.
        let now = get_time();
        let absolute_frame_number = (now * 1000.0 / config.ms_per_animation_frame) as u32;
        let time_since_last_frame = now - last_frame_time;

        last_frame_time = now;

        // If the player isn't mostly inside the current level, change levels.
        {
            if !level.contains_majority_of(&player.bbox()) {
                let world_pos = level.to_world_coords(&player.pos);
                if let Some((new_level, new_pos)) =
                    world.find_level_containing_majority_of(&world_pos, &player.relative_bbox)
                {
                    level = new_level;
                    player.pos = new_pos;
                }
            }
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

        run_manager.update(
            &config,
            time_since_last_frame,
            is_key_down(KeyCode::A),
            is_key_down(KeyCode::D),
        );
        let mut x_impulse: f32 = 0.;

        if player.is_in_air {
            if is_key_down(KeyCode::Space) && player.velocity.y < 0. {
                player.velocity.y -=
                    config.long_jump_keypress_extra_force * time_since_last_frame as f32;
            }
            player.velocity.y += config.gravity * time_since_last_frame as f32;
            if run_manager.is_running() {
                player.velocity.x = run_manager.run_speed();
            }
        } else {
            if is_key_pressed(KeyCode::Space) {
                player.velocity = Vec2::new(run_manager.run_speed(), -config.jump_velocity);
                player.is_in_air = true
            } else {
                x_impulse = run_manager.run_speed();
            }
        }

        let player_prev_bbox = player.bbox();
        player.pos.x += (player.velocity.x + x_impulse) * time_since_last_frame as f32;
        player.pos.y += player.velocity.y * time_since_last_frame as f32;

        let mut is_on_any_surface_this_frame = false;

        loop {
            let player_actor = Actor {
                prev_bbox: player_prev_bbox,
                bbox: player.bbox(),
                velocity: player.velocity,
            };
            let mut displacement_occurred = false;
            for collider in level.iter_colliders(&player_actor.bbox) {
                if let Some(collision) = process_collision(&collider, &player_actor) {
                    if collision.is_on_surface {
                        is_on_any_surface_this_frame = true;
                    }
                    if let Some(new_velocity) = collision.new_velocity {
                        player.velocity = new_velocity;
                    }

                    if collision.displacement.x != 0. || collision.displacement.y != 0. {
                        player.pos += collision.displacement;
                        displacement_occurred = true;
                        break;
                    }
                }
            }
            if !displacement_occurred {
                break;
            }
        }

        if is_on_any_surface_this_frame {
            // The player just landed (or remains on the ground).
            player.is_in_air = false;
        } else if !player.is_in_air {
            // The player just fell off a ledge.
            player.is_in_air = true;
        }

        // Draw NPCs.

        for flying_eye in level.iter_flying_eyes() {
            let sprite = &sprites.flying_eye.flight;
            let bbox = &sprites.flying_eye.flight_bbox;
            let sprite_pos = flying_eye.point() - bbox.point();
            sprite.draw(
                sprite_pos.x,
                sprite_pos.y,
                absolute_frame_number % sprite.num_frames(),
            );
            if debug_mode {
                sprite.draw_debug_rect(flying_eye.x, flying_eye.y, GREEN);
                let flying_eye_bbox = bbox.offset(sprite_pos);
                draw_rect_lines(&flying_eye_bbox, 2., PURPLE);
            }
        }

        // Draw player.

        let sprite: &Sprite;

        if player.is_in_air {
            if player.velocity.y >= 0. {
                sprite = &sprites.huntress.fall;
            } else {
                sprite = &sprites.huntress.jump;
            }
        } else {
            if x_impulse != 0. {
                sprite = &sprites.huntress.run;
                player.is_facing_left = x_impulse < 0.;
            } else {
                sprite = &sprites.huntress.idle;
            }
        }

        sprite.draw_ex(
            player.pos.x,
            player.pos.y,
            absolute_frame_number % sprite.num_frames(),
            SpriteDrawParams {
                flip_x: player.is_facing_left,
                ..Default::default()
            },
        );

        // Draw level text.
        let player_bbox = player.bbox();
        if let Some(text) = level.get_text(&player_bbox) {
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
            sprite.draw_debug_rect(player.pos.x, player.pos.y, GREEN);
            draw_rect_lines(&player_bbox, 2., PURPLE);
            for collider in level.iter_colliders(&level.pixel_bounds()) {
                collider.draw_debug_rect(PURPLE);
            }
            draw_rect_lines(&level.get_bounding_cell_rect(&player_bbox), 1., WHITE);
            let text = format!("fps: {}", get_fps());
            draw_text(&text, camera_rect.x + 32., camera_rect.y + 32., 32.0, WHITE);
        }

        // Wait for the next frame.

        next_frame().await;
    }
}
