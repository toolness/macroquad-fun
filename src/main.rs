use macroquad::prelude::*;
use sprite::{Sprite, SpriteDrawParams};

mod sprite;

const SCALE: f32 = 3.;

const MS_PER_ANIMATION_FRAME: f64 = 100.0;

const RUN_SPEED: f64 = 300.0;

const IDLE_FRAME_FEET_Y: f32 = 96.0;

const GROUND_HEIGHT: f32 = 8.0 * SCALE;

const GRAVITY: f32 = 1500.0;

const JUMP_VELOCITY: f32 = 600.0;

struct GameSprites {
    idle: Sprite,
    run: Sprite,
    jump: Sprite,
    fall: Sprite,
}

#[macroquad::main("Fun")]
async fn main() {
    let sprites = GameSprites {
        idle: Sprite::new(
            load_texture("media/Huntress/Sprites/Idle.png")
                .await
                .unwrap(),
            8,
            SCALE,
        ),
        run: Sprite::new(
            load_texture("media/Huntress/Sprites/Run.png")
                .await
                .unwrap(),
            8,
            SCALE,
        ),
        jump: Sprite::new(
            load_texture("media/Huntress/Sprites/Jump.png")
                .await
                .unwrap(),
            2,
            SCALE,
        ),
        fall: Sprite::new(
            load_texture("media/Huntress/Sprites/Fall.png")
                .await
                .unwrap(),
            2,
            SCALE,
        ),
    };
    let mut x = screen_width() / 2. - sprites.idle.frame_width() / 2.0;
    let ground_y = screen_height() - GROUND_HEIGHT;
    let sprite_ground_y = ground_y - IDLE_FRAME_FEET_Y * SCALE;
    let mut y = sprite_ground_y;
    let mut is_in_air = false;
    let mut velocity = Vec2::new(0., 0.);
    let mut last_frame_time = get_time();
    let mut is_facing_left = false;
    let mut debug_mode = false;

    loop {
        let now = get_time();
        let absolute_frame_number = (now * 1000.0 / MS_PER_ANIMATION_FRAME) as u32;
        let time_since_last_frame = now - last_frame_time;

        last_frame_time = now;

        clear_background(GRAY);

        if is_key_released(KeyCode::Escape) {
            break;
        }

        draw_rectangle(0., ground_y, screen_width(), GROUND_HEIGHT, DARKGRAY);

        let is_pressing_right = is_key_down(KeyCode::D);
        let is_pressing_left = is_key_down(KeyCode::A);

        if is_key_pressed(KeyCode::Space) && !is_in_air {
            let x_velocity = if is_pressing_left {
                -RUN_SPEED as f32
            } else if is_pressing_right {
                RUN_SPEED as f32
            } else {
                0.
            };
            velocity = Vec2::new(x_velocity, -JUMP_VELOCITY);
            is_in_air = true
        } else if is_in_air {
            if y >= sprite_ground_y {
                is_in_air = false;
                velocity = Vec2::new(0., 0.);
            } else {
                velocity.y += GRAVITY * time_since_last_frame as f32;
            }
        }

        x += velocity.x * time_since_last_frame as f32;
        y += velocity.y * time_since_last_frame as f32;

        let sprite: &Sprite;

        if is_in_air {
            if velocity.y >= 0. {
                sprite = &sprites.fall;
            } else {
                sprite = &sprites.jump;
            }
        } else {
            if is_pressing_left || is_pressing_right {
                sprite = &sprites.run;
                let run_amount = (time_since_last_frame * RUN_SPEED) as f32;
                if is_pressing_right {
                    is_facing_left = false;
                    x += run_amount;
                } else {
                    is_facing_left = true;
                    x -= run_amount;
                }
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

        if is_key_pressed(KeyCode::GraveAccent) {
            debug_mode = !debug_mode;
        }
        if debug_mode {
            sprite.draw_debug_rect(x, y, GREEN);
            let text = format!("fps: {}", get_fps());
            draw_text(&text, 32., 32., 32.0, WHITE);
        }

        next_frame().await;
    }
}
