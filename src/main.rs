extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use level::Level;
use macroquad::prelude::*;
use sprite::{Sprite, SpriteDrawParams};

use crate::collision::{process_collision, Actor};

mod collision;
mod ldtk;
mod level;
mod sprite;

const SPRITE_SCALE: f32 = 3.;

const MS_PER_ANIMATION_FRAME: f64 = 100.0;

const RUN_SPEED: f64 = 300.0;

const IDLE_FRAME_HEAD_Y: f32 = 58.0;

const IDLE_FRAME_FEET_Y: f32 = 96.0;

const IDLE_FRAME_LEFT_X: f32 = 64.0;

const IDLE_FRAME_RIGHT_X: f32 = 83.0;

const GRAVITY: f32 = 1500.0;

const JUMP_VELOCITY: f32 = 900.0;

struct GameSprites {
    idle: Sprite,
    run: Sprite,
    jump: Sprite,
    fall: Sprite,
}

#[macroquad::main("Fun")]
async fn main() {
    let level = Level::load("media/world.ldtk", SPRITE_SCALE).unwrap();

    request_new_screen_size(level.width_in_pixels(), level.height_in_pixels());
    next_frame().await;

    let sprites = GameSprites {
        idle: Sprite::new(
            load_texture("media/Huntress/Sprites/Idle.png")
                .await
                .unwrap(),
            8,
            SPRITE_SCALE,
        ),
        run: Sprite::new(
            load_texture("media/Huntress/Sprites/Run.png")
                .await
                .unwrap(),
            8,
            SPRITE_SCALE,
        ),
        jump: Sprite::new(
            load_texture("media/Huntress/Sprites/Jump.png")
                .await
                .unwrap(),
            2,
            SPRITE_SCALE,
        ),
        fall: Sprite::new(
            load_texture("media/Huntress/Sprites/Fall.png")
                .await
                .unwrap(),
            2,
            SPRITE_SCALE,
        ),
    };
    let environment: Vec<Rect> = level.get_all_rects();
    let player_relative_bbox = Rect::new(
        IDLE_FRAME_LEFT_X * SPRITE_SCALE,
        IDLE_FRAME_HEAD_Y * SPRITE_SCALE,
        (IDLE_FRAME_RIGHT_X - IDLE_FRAME_LEFT_X) * SPRITE_SCALE,
        (IDLE_FRAME_FEET_Y - IDLE_FRAME_HEAD_Y) * SPRITE_SCALE,
    );
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
        let absolute_frame_number = (now * 1000.0 / MS_PER_ANIMATION_FRAME) as u32;
        let time_since_last_frame = now - last_frame_time;

        last_frame_time = now;

        // Draw environment.

        clear_background(GRAY);
        level.draw();

        // Process input/physics.

        let is_pressing_right = is_key_down(KeyCode::D);
        let is_pressing_left = is_key_down(KeyCode::A);
        let run_velocity = if is_pressing_left {
            -RUN_SPEED
        } else if is_pressing_right {
            RUN_SPEED
        } else {
            0.
        } as f32;
        let mut x_impulse: f32 = 0.;

        if is_in_air {
            velocity.y += GRAVITY * time_since_last_frame as f32;
            if run_velocity != 0. {
                velocity.x = run_velocity;
            }
        } else {
            if is_key_pressed(KeyCode::Space) {
                velocity = Vec2::new(run_velocity, -JUMP_VELOCITY);
                is_in_air = true
            } else {
                x_impulse = run_velocity;
            }
        }

        let player_prev_bbox = player_relative_bbox.offset(Vec2::new(x, y));
        x += (velocity.x + x_impulse) * time_since_last_frame as f32;
        y += velocity.y * time_since_last_frame as f32;

        let mut is_on_any_surface_this_frame = false;

        for collider in environment.iter() {
            let player_actor = Actor {
                prev_bbox: player_prev_bbox,
                bbox: player_relative_bbox.offset(Vec2::new(x, y)),
                velocity,
            };
            if let Some(collision) = process_collision(&collider, &player_actor) {
                if collision.is_on_surface {
                    is_on_any_surface_this_frame = true;
                }
                if let Some(new_velocity) = collision.new_velocity {
                    velocity = new_velocity;
                }

                x += collision.displacement.x;
                y += collision.displacement.y;
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
            draw_debug_collision_rect(&player_relative_bbox.offset(Vec2::new(x, y)));
            for collider in environment.iter() {
                draw_debug_collision_rect(&collider);
            }
            let text = format!("fps: {}", get_fps());
            draw_text(&text, 32., 32., 32.0, WHITE);
        }

        // Wait for the next frame.

        next_frame().await;
    }
}

fn draw_debug_collision_rect(collider: &Rect) {
    draw_rectangle_lines(
        collider.left(),
        collider.top(),
        collider.size().x,
        collider.size().y,
        2.,
        PURPLE,
    );
}
