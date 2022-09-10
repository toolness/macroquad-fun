use macroquad::prelude::{Rect, Vec2, GREEN, PURPLE};

use crate::{
    drawing::draw_rect_lines,
    sprite::{Sprite, SpriteDrawParams},
};

#[derive(Default)]
pub struct SpriteEntity {
    pub pos: Vec2,
    pub relative_bbox: Rect,
    pub sprite: Option<&'static Sprite>,
    pub is_facing_left: bool,
}

impl SpriteEntity {
    pub fn bbox(&self) -> Rect {
        self.relative_bbox.offset(self.pos)
    }

    pub fn draw(&self, absolute_frame_number: u32) {
        if let Some(sprite) = self.sprite {
            sprite.draw_ex(
                self.pos.x,
                self.pos.y,
                absolute_frame_number % sprite.num_frames(),
                SpriteDrawParams {
                    flip_x: self.is_facing_left,
                    ..Default::default()
                },
            );
        }
    }

    pub fn draw_debug_rects(&self) {
        if let Some(sprite) = self.sprite {
            sprite.draw_debug_rect(self.pos.x, self.pos.y, GREEN);
        }
        draw_rect_lines(&self.bbox(), 2., PURPLE);
    }
}