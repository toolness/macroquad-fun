use macroquad::prelude::*;
use sprite::Sprite;

mod sprite;

const SCALE: f32 = 3.;

const MS_PER_ANIMATION_FRAME: f64 = 100.0;

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
    let x = screen_width() / 2. - sprites.idle.frame_width() / 2.0;
    let y = screen_height() / 2. - sprites.idle.frame_height() / 2.;

    loop {
        let absolute_frame_number = (get_time() * 1000.0 / MS_PER_ANIMATION_FRAME) as u32;

        clear_background(GRAY);

        if is_key_released(KeyCode::Escape) {
            break;
        }
        if is_key_down(KeyCode::D) {
            sprites.run.draw(
                x,
                y,
                WHITE,
                absolute_frame_number % sprites.run.num_frames(),
            );
        } else {
            sprites.idle.draw(
                x,
                y,
                WHITE,
                absolute_frame_number % sprites.idle.num_frames(),
            );
        }
        next_frame().await;
    }
}
