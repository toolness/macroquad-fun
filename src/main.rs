use macroquad::prelude::*;
use sprite::{Sprite, SpriteDrawParams};

mod sprite;

const SPRITE_SCALE: f32 = 3.;

const MS_PER_ANIMATION_FRAME: f64 = 100.0;

const RUN_SPEED: f64 = 300.0;

const IDLE_FRAME_HEAD_Y: f32 = 58.0;

const IDLE_FRAME_FEET_Y: f32 = 96.0;

const IDLE_FRAME_LEFT_X: f32 = 64.0;

const IDLE_FRAME_RIGHT_X: f32 = 83.0;

const GROUND_HEIGHT: f32 = 8.0 * SPRITE_SCALE;

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
    let mut x = screen_width() / 2. - sprites.idle.frame_width() / 2.0;
    let ground_y = screen_height() - GROUND_HEIGHT;
    let sprite_ground_y = ground_y - IDLE_FRAME_FEET_Y * SPRITE_SCALE;
    let mut y = sprite_ground_y;
    let mut is_in_air = false;
    let mut velocity = Vec2::new(0., 0.);
    let mut last_frame_time = get_time();
    let mut is_facing_left = false;
    let mut debug_mode = false;
    let environment: Vec<Rect> = vec![Rect::new(0., ground_y, screen_width(), GROUND_HEIGHT)];
    let mut player_relative_bbox = Rect::new(
        IDLE_FRAME_LEFT_X * SPRITE_SCALE,
        IDLE_FRAME_HEAD_Y * SPRITE_SCALE,
        IDLE_FRAME_RIGHT_X - IDLE_FRAME_LEFT_X,
        IDLE_FRAME_FEET_Y - IDLE_FRAME_HEAD_Y,
    );
    player_relative_bbox.scale(SPRITE_SCALE, SPRITE_SCALE);

    loop {
        // Keep track of time.
        let now = get_time();
        let absolute_frame_number = (now * 1000.0 / MS_PER_ANIMATION_FRAME) as u32;
        let time_since_last_frame = now - last_frame_time;

        last_frame_time = now;

        // Draw environment.

        clear_background(GRAY);
        for collider in environment.iter() {
            draw_rectangle(
                collider.left(),
                collider.top(),
                collider.size().x,
                collider.size().y,
                DARKGRAY,
            );
        }

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

        if is_in_air {
            if y >= sprite_ground_y {
                is_in_air = false;
                velocity = Vec2::new(0., 0.);
                y = sprite_ground_y;
            } else {
                velocity.y += GRAVITY * time_since_last_frame as f32;
                if run_velocity != 0. {
                    velocity.x = run_velocity;
                }
            }
        } else {
            if is_key_pressed(KeyCode::Space) {
                velocity = Vec2::new(run_velocity, -JUMP_VELOCITY);
                is_in_air = true
            } else {
                x += run_velocity * time_since_last_frame as f32;
            }
        }

        x += velocity.x * time_since_last_frame as f32;
        y += velocity.y * time_since_last_frame as f32;

        // Draw player.

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
                is_facing_left = is_pressing_left;
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
            draw_debug_collision_rect(&player_bbox);
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
