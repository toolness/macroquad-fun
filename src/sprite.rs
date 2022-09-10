use macroquad::prelude::*;

use crate::config::config;

pub struct Sprite {
    texture: Texture2D,
    scale: f32,
    frame_size: Vec2,
    num_frames: u32,
}

pub struct SpriteDrawParams {
    /// Mirror on the X axis
    pub flip_x: bool,

    /// Mirror on the Y axis
    pub flip_y: bool,

    pub color: Color,
}

impl Default for SpriteDrawParams {
    fn default() -> Self {
        Self {
            flip_x: false,
            flip_y: false,
            color: WHITE,
        }
    }
}

impl Sprite {
    pub fn new(texture: Texture2D, num_frames: u32) -> Self {
        texture.set_filter(FilterMode::Nearest);
        Sprite {
            texture,
            num_frames,
            frame_size: Vec2::new(texture.width() / num_frames as f32, texture.height()),
            scale: config().sprite_scale,
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

    pub fn draw_ex(&self, x: f32, y: f32, frame_number: u32, params: SpriteDrawParams) {
        draw_texture_ex(
            self.texture,
            x,
            y,
            params.color,
            DrawTextureParams {
                flip_x: params.flip_x,
                flip_y: params.flip_y,
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

    #[allow(dead_code)]
    pub fn draw(&self, x: f32, y: f32, frame_number: u32) {
        self.draw_ex(x, y, frame_number, Default::default())
    }

    pub fn draw_debug_rect(&self, x: f32, y: f32, color: Color) {
        draw_rectangle_lines(x, y, self.frame_width(), self.frame_height(), 1.0, color)
    }
}
