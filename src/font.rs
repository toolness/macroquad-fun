// Most of this logic is taken from pman-sdl's font-rendering code:
//
// https://github.com/toolness/pman-sdl/blob/master/src/font.h
use macroquad::{
    prelude::{Color, Rect, Vec2},
    texture::{draw_texture_ex, DrawTextureParams, Texture2D},
};

use crate::config::config;

/// ASCII value to subtract from every ASCII character we're asked to print.
const CHAR_CODE_OFFSET: u32 = 32;

pub struct BitmapFont {
    pub texture: Texture2D,
    pub char_width: u32,
    pub char_height: u32,
    pub chars_per_line: u32,
}

impl BitmapFont {
    pub fn draw_text<T: AsRef<str>>(&self, text: T, x: f32, y: f32, color: Color) {
        let scale = config().sprite_scale;
        let scaled_size = Vec2::new(
            self.char_width as f32 * scale,
            self.char_height as f32 * scale,
        );
        let mut curr_x = x.floor();
        let curr_y = y.floor();
        for char in text.as_ref().chars() {
            if char.is_ascii() {
                let char_code = char as u32 - CHAR_CODE_OFFSET;
                let char_x = char_code % self.chars_per_line;
                let char_y = char_code / self.chars_per_line;
                let source_x = char_x * self.char_width;
                let source_y = char_y * self.char_height;
                let source = Some(Rect::new(
                    source_x as f32,
                    source_y as f32,
                    self.char_width as f32,
                    self.char_height as f32,
                ));
                draw_texture_ex(
                    self.texture,
                    curr_x,
                    curr_y,
                    color,
                    DrawTextureParams {
                        dest_size: Some(scaled_size),
                        source,
                        ..Default::default()
                    },
                );
                curr_x += scaled_size.x as f32;
            }
        }
    }
}
