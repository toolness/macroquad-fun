use macroquad::prelude::*;
use sprite::{Sprite, SpriteDrawParams};

mod sprite;

const SCALE: f32 = 3.;

const MS_PER_ANIMATION_FRAME: f64 = 100.0;

const RUN_SPEED: f64 = 300.0;

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
    let y = screen_height() / 2. - sprites.idle.frame_height() / 2.;
    let mut last_frame_time = get_time();
    let mut is_facing_left = false;

    loop {
        let now = get_time();
        let absolute_frame_number = (now * 1000.0 / MS_PER_ANIMATION_FRAME) as u32;
        let time_since_last_frame = now - last_frame_time;

        last_frame_time = now;

        clear_background(GRAY);

        if is_key_released(KeyCode::Escape) {
            break;
        }

        let sprite: &Sprite;

        if is_key_down(KeyCode::D) || is_key_down(KeyCode::A) {
            sprite = &sprites.run;
            let run_amount = (time_since_last_frame * RUN_SPEED) as f32;
            if is_key_down(KeyCode::D) {
                is_facing_left = false;
                x += run_amount;
            } else {
                is_facing_left = true;
                x -= run_amount;
            }
        } else if is_key_down(KeyCode::W) {
            sprite = &sprites.jump;
        } else if is_key_down(KeyCode::S) {
            sprite = &sprites.fall;
        } else {
            sprite = &sprites.idle;
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

        next_frame().await;
    }
}
