extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use aseprite::load_aseprite_slices;
use config::load_config;
use drawing::draw_rect_lines;
use level::Level;
use macroquad::prelude::*;
use sprite::{Sprite, SpriteDrawParams};

use crate::collision::{process_collision, Actor};

mod aseprite;
mod collision;
mod config;
mod drawing;
mod ldtk;
mod level;
mod sprite;

struct GameSprites {
    idle: Sprite,
    run: Sprite,
    jump: Sprite,
    fall: Sprite,
}

#[macroquad::main("Fun")]
async fn main() {
    let config = load_config("media/config.json").await.unwrap();
    let sprite_scale = config.sprite_scale;
    let idle_slices = load_aseprite_slices("media/Huntress/Sprites/Idle.json", sprite_scale)
        .await
        .unwrap();
    let player_relative_bbox = idle_slices.get("idle_bounding_box").unwrap();
    let level = Level::load("media/world.ldtk", sprite_scale).await.unwrap();

    request_new_screen_size(level.width_in_pixels(), level.height_in_pixels());
    next_frame().await;

    let sprites = GameSprites {
        idle: Sprite::new(
            load_texture("media/Huntress/Sprites/Idle.png")
                .await
                .unwrap(),
            8,
            sprite_scale,
        ),
        run: Sprite::new(
            load_texture("media/Huntress/Sprites/Run.png")
                .await
                .unwrap(),
            8,
            sprite_scale,
        ),
        jump: Sprite::new(
            load_texture("media/Huntress/Sprites/Jump.png")
                .await
                .unwrap(),
            2,
            sprite_scale,
        ),
        fall: Sprite::new(
            load_texture("media/Huntress/Sprites/Fall.png")
                .await
                .unwrap(),
            2,
            sprite_scale,
        ),
    };
    let player_start_bottom_left = level.player_start_bottom_left_in_pixels();
    let mut x = player_start_bottom_left.x - player_relative_bbox.x;
    let mut y = player_start_bottom_left.y - player_relative_bbox.bottom();
    let mut is_in_air = false;
    let mut velocity = Vec2::new(0., 0.);
    let mut last_frame_time = get_time();
    let mut is_facing_left = false;
    let mut debug_mode = false;

    loop {
        // Keep track of time.
        let now = get_time();
        let absolute_frame_number = (now * 1000.0 / config.ms_per_animation_frame) as u32;
        let time_since_last_frame = now - last_frame_time;

        last_frame_time = now;

        // Draw environment.

        clear_background(GRAY);
        level.draw();

        // Process input/physics.

        let is_pressing_right = is_key_down(KeyCode::D);
        let is_pressing_left = is_key_down(KeyCode::A);
        let x_direction = if is_pressing_left {
            -1.
        } else if is_pressing_right {
            1.
        } else {
            0.
        } as f32;
        let mut x_impulse: f32 = 0.;

        if is_in_air {
            velocity.y += config.gravity * time_since_last_frame as f32;
            if x_direction != 0. {
                velocity.x = config.run_speed * x_direction;
            }
        } else {
            if is_key_pressed(KeyCode::Space) {
                velocity = Vec2::new(config.run_speed * x_direction, -config.jump_velocity);
                is_in_air = true
            } else {
                x_impulse = config.run_speed * x_direction;
            }
        }

        let player_prev_bbox = player_relative_bbox.offset(Vec2::new(x, y));
        x += (velocity.x + x_impulse) * time_since_last_frame as f32;
        y += velocity.y * time_since_last_frame as f32;

        let mut is_on_any_surface_this_frame = false;

        loop {
            let player_actor = Actor {
                prev_bbox: player_prev_bbox,
                bbox: player_relative_bbox.offset(Vec2::new(x, y)),
                velocity,
            };
            let mut displacement_occurred = false;
            for collider in level.iter_colliders(&player_actor.bbox) {
                if let Some(collision) = process_collision(&collider, &player_actor) {
                    if collision.is_on_surface {
                        is_on_any_surface_this_frame = true;
                    }
                    if let Some(new_velocity) = collision.new_velocity {
                        velocity = new_velocity;
                    }

                    if collision.displacement.x != 0. || collision.displacement.y != 0. {
                        x += collision.displacement.x;
                        y += collision.displacement.y;
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
            is_in_air = false;
        } else if !is_in_air {
            // The player just fell off a ledge.
            is_in_air = true;
        }

        // Draw player.

        let sprite: &Sprite;

        if is_in_air {
            if velocity.y >= 0. {
                sprite = &sprites.fall;
            } else {
                sprite = &sprites.jump;
            }
        } else {
            if x_impulse != 0. {
                sprite = &sprites.run;
                is_facing_left = x_impulse < 0.;
            } else {
                sprite = &sprites.idle;
            }
        }

        sprite.draw_ex(
            x,
            y,
            absolute_frame_number % sprite.num_frames(),
            SpriteDrawParams {
                flip_x: is_facing_left,
                ..Default::default()
            },
        );

        // Process miscellaneous system input.

        if is_key_released(KeyCode::Escape) {
            break;
        } else if is_key_pressed(KeyCode::GraveAccent) {
            debug_mode = !debug_mode;
        }
        if debug_mode {
            sprite.draw_debug_rect(x, y, GREEN);
            let player_bbox = player_relative_bbox.offset(Vec2::new(x, y));
            draw_rect_lines(&player_bbox, 2., PURPLE);
            for collider in level.iter_colliders(&Rect::new(
                0.,
                0.,
                level.width_in_pixels(),
                level.height_in_pixels(),
            )) {
                collider.draw_debug_rect(PURPLE);
            }
            draw_rect_lines(&level.get_bounding_cell_rect(&player_bbox), 1., WHITE);
            let text = format!("fps: {}", get_fps());
            draw_text(&text, 32., 32., 32.0, WHITE);
        }

        // Wait for the next frame.

        next_frame().await;
    }
}
