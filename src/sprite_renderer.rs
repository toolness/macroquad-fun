use macroquad::prelude::*;

use crate::config::config;

pub struct SpriteRenderer {
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

    pub rotation: f32,

    pub color: Color,

    /// Rotate around this point.
    /// When `None`, rotate around the texture's center.
    /// When `Some`, the coordinates are in screen-space.
    /// E.g. pivot (0,0) rotates around the top left corner of the screen, not of the
    /// texture.
    pub pivot: Option<Vec2>,
}

impl Default for SpriteDrawParams {
    fn default() -> Self {
        Self {
            flip_x: false,
            flip_y: false,
            rotation: 0.,
            color: WHITE,
            pivot: None,
        }
    }
}

impl SpriteRenderer {
    pub fn new(texture: Texture2D, num_frames: u32) -> Self {
        texture.set_filter(FilterMode::Nearest);
        SpriteRenderer {
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

    pub fn last_frame(&self) -> u32 {
        self.num_frames - 1
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
                rotation: params.rotation,
                dest_size: Some(self.frame_size * self.scale),
                source: Some(Rect {
                    x: self.frame_size.x * frame_number as f32,
                    y: 0.,
                    w: self.frame_size.x,
                    h: self.frame_size.y,
                }),
                pivot: params.pivot,
            },
        )
    }

    #[allow(dead_code)]
    pub fn draw(&self, x: f32, y: f32, frame_number: u32) {
        self.draw_ex(x, y, frame_number, Default::default())
    }
}
