use macroquad::prelude::*;
use sprite::Sprite;

mod sprite;

const NUM_IDLE_FRAMES: u32 = 8;

const SCALE: f32 = 3.;

const MS_PER_ANIMATION_FRAME: f64 = 100.0;

#[macroquad::main("Fun")]
async fn main() {
    let idle_sprite = Sprite::new(
        load_texture("media/Huntress/Sprites/Idle.png")
            .await
            .unwrap(),
        NUM_IDLE_FRAMES,
        SCALE,
    );
    let x = screen_width() / 2. - idle_sprite.frame_width() / 2.0;
    let y = screen_height() / 2. - idle_sprite.frame_height() / 2.;

    loop {
        let absolute_frame_number = (get_time() * 1000.0 / MS_PER_ANIMATION_FRAME) as u32;

        clear_background(GRAY);

        idle_sprite.draw(
            x,
            y,
            WHITE,
            absolute_frame_number % idle_sprite.num_frames(),
        );
        next_frame().await;
    }
}
