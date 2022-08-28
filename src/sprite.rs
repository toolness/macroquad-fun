use macroquad::prelude::*;

pub struct Sprite {
    texture: Texture2D,
    scale: f32,
    frame_size: Vec2,
    num_frames: u32,
}

impl Sprite {
    pub fn new(texture: Texture2D, num_frames: u32, scale: f32) -> Self {
        texture.set_filter(FilterMode::Nearest);
        Sprite {
            texture,
            num_frames,
            frame_size: Vec2::new(texture.width() / num_frames as f32, texture.height()),
            scale,
        }
    }

    pub fn frame_width(&self) -> f32 {
        self.frame_size.x * self.scale
    }

    pub fn frame_height(&self) -> f32 {
        self.frame_size.y * self.scale
    }

    pub fn num_frames(&self) -> u32 {
        self.num_frames
    }

    pub fn draw(&self, x: f32, y: f32, color: Color, frame_number: u32) {
        draw_texture_ex(
            self.texture,
            x,
            y,
            color,
            DrawTextureParams {
                dest_size: Some(self.frame_size * self.scale),
                source: Some(Rect {
                    x: self.frame_size.x * frame_number as f32,
                    y: 0.,
                    w: self.frame_size.x,
                    h: self.frame_size.y,
                }),
                ..Default::default()
            },
        )
    }
}
