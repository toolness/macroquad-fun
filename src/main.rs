use macroquad::prelude::*;
use sprite::Sprite;

mod sprite;

const NUM_IDLE_FRAMES: u32 = 8;
const SCALE: f32 = 3.;

#[macroquad::main("Fun")]
async fn main() {
    let idle_sprite = Sprite::new(
        load_texture("media/Huntress/Sprites/Idle.png")
            .await
            .unwrap(),
        NUM_IDLE_FRAMES,
        SCALE,
    );
    let mut idle_frame_num = 0;
    let mut frame_number = 0;
    let x = screen_width() / 2. - idle_sprite.frame_width() / 2.0;
    let y = screen_height() / 2. - idle_sprite.frame_height() / 2.;

    loop {
        clear_background(GRAY);

        idle_sprite.draw(x, y, WHITE, idle_frame_num);
        next_frame().await;

        frame_number += 1;
        if frame_number % 10 == 0 {
            frame_number = 0;
            idle_frame_num = (idle_frame_num + 1) % idle_sprite.num_frames();
        }
    }
}
