use macroquad::prelude::*;
use sprite::{Sprite, SpriteDrawParams};

mod sprite;

const SCALE: f32 = 3.;

const MS_PER_ANIMATION_FRAME: f64 = 100.0;

const RUN_SPEED: f64 = 300.0;

struct GameSprites {
    idle: Sprite,
    run: Sprite,
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
    };
    let mut x = screen_width() / 2. - sprites.idle.frame_width() / 2.0;
    let y = screen_height() / 2. - sprites.idle.frame_height() / 2.;
    let mut last_frame_time = get_time();

    loop {
        let now = get_time();
        let absolute_frame_number = (now * 1000.0 / MS_PER_ANIMATION_FRAME) as u32;
        let time_since_last_frame = now - last_frame_time;

        last_frame_time = now;

        clear_background(GRAY);

        if is_key_released(KeyCode::Escape) {
            break;
        }
        if is_key_down(KeyCode::D) {
            sprites
                .run
                .draw(x, y, absolute_frame_number % sprites.run.num_frames());
            x += (time_since_last_frame * RUN_SPEED) as f32;
        } else if is_key_down(KeyCode::A) {
            sprites.run.draw_ex(
                x,
                y,
                absolute_frame_number % sprites.run.num_frames(),
                SpriteDrawParams {
                    flip_x: true,
                    ..Default::default()
                },
            );
            x -= (time_since_last_frame * RUN_SPEED) as f32;
        } else {
            sprites
                .idle
                .draw(x, y, absolute_frame_number % sprites.idle.num_frames());
        }
        next_frame().await;
    }
}
