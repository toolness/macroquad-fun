use macroquad::prelude::*;

const NUM_IDLE_FRAMES: u32 = 8;
const SCALE: f32 = 3.;

#[macroquad::main("Fun")]
async fn main() {
    let texture: Texture2D = load_texture("media/Huntress/Sprites/Idle.png")
        .await
        .unwrap();
    let idle_frame_size: Vec2 =
        Vec2::new(texture.width() / NUM_IDLE_FRAMES as f32, texture.height());
    let mut idle_frame_num = 0;
    let mut frame_number = 0;

    texture.set_filter(FilterMode::Nearest);

    loop {
        clear_background(GRAY);
        draw_texture_ex(
            texture,
            screen_width() / 2. - idle_frame_size.x * SCALE / 2.0,
            screen_height() / 2. - idle_frame_size.y * SCALE / 2.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(idle_frame_size * SCALE),
                source: Some(Rect {
                    x: idle_frame_size.x * idle_frame_num as f32,
                    y: 0.,
                    w: idle_frame_size.x,
                    h: idle_frame_size.y,
                }),
                ..Default::default()
            },
        );
        next_frame().await;

        frame_number += 1;
        if frame_number % 10 == 0 {
            frame_number = 0;
            idle_frame_num = (idle_frame_num + 1) % NUM_IDLE_FRAMES;
        }
    }
}
