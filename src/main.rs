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

const PLATFORM_WIDTH: f32 = 64.0 * SPRITE_SCALE;

const PLATFORM_HEIGHT: f32 = 8.0 * SPRITE_SCALE;

const PLATFORM_DISTANCE_FROM_GROUND: f32 = 64.0 * SPRITE_SCALE;

const WALL_WIDTH: f32 = 8.0 * SPRITE_SCALE;

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
    let environment: Vec<Rect> = vec![
        // Ground
        Rect::new(
            0.,
            screen_height() - GROUND_HEIGHT,
            screen_width(),
            GROUND_HEIGHT,
        ),
        // Left wall
        Rect::new(0., 0., WALL_WIDTH, screen_height()),
        // Right wall
        Rect::new(screen_width() - WALL_WIDTH, 0., WALL_WIDTH, screen_height()),
        // Platform
        Rect::new(
            0.,
            screen_height() - GROUND_HEIGHT - PLATFORM_DISTANCE_FROM_GROUND,
            PLATFORM_WIDTH,
            PLATFORM_HEIGHT,
        ),
    ];
    let player_relative_bbox = Rect::new(
        IDLE_FRAME_LEFT_X * SPRITE_SCALE,
        IDLE_FRAME_HEAD_Y * SPRITE_SCALE,
        (IDLE_FRAME_RIGHT_X - IDLE_FRAME_LEFT_X) * SPRITE_SCALE,
        (IDLE_FRAME_FEET_Y - IDLE_FRAME_HEAD_Y) * SPRITE_SCALE,
    );
    let mut x = screen_width() / 2. - sprites.idle.frame_width() / 2.0;
    let mut y = environment[0].top() - player_relative_bbox.bottom();
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

        let velocity_this_frame = Vec2::new(velocity.x + x_impulse, velocity.y);
        x += velocity_this_frame.x * time_since_last_frame as f32;
        y += velocity_this_frame.y * time_since_last_frame as f32;

        let player_bbox = player_relative_bbox.offset(Vec2::new(x, y));
        for collider in environment.iter() {
            if let Some(intersection) = collider.intersect(player_bbox) {
                if intersection.top() <= collider.top() {
                    // The top of the collider is being intersected with.
                    is_in_air = false;
                    velocity = Vec2::new(0., 0.);
                    let y_diff = player_bbox.bottom() - collider.top();
                    y -= y_diff;
                } else if intersection.bottom() >= collider.bottom() {
                    // The bottom side of the collider is being intersected with.
                    velocity.y = 0.;
                    let y_diff = collider.bottom() - player_bbox.top();
                    y += y_diff;
                } else if intersection.left() <= collider.left() {
                    // The left side of the collider is being intersected with.
                    let x_diff = player_bbox.right() - collider.left();
                    x -= x_diff;
                } else if intersection.right() >= collider.right() {
                    // The right side of the collider is being intersected with.
                    let x_diff = collider.right() - player_bbox.left();
                    x += x_diff;
                }
            }
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
