use macroquad::prelude::{Rect, Vec2, GREEN, PURPLE};

use crate::{
    drawing::draw_rect_lines,
    sprite_renderer::{SpriteDrawParams, SpriteRenderer},
    time::GameTime,
};

#[derive(Default)]
pub struct SpriteComponent {
    pub pos: Vec2,
    pub relative_bbox: Rect,
    pub renderer: Option<&'static SpriteRenderer>,
    pub is_facing_left: bool,
    pub flip_bbox_when_facing_left: bool,
    pub current_frame_number: u32,
}

impl SpriteComponent {
    pub fn bbox(&self) -> Rect {
        if self.flip_bbox_when_facing_left && self.is_facing_left {
            if let Some(sprite) = self.renderer {
                let center_offset = sprite.frame_width() / 2. - self.relative_bbox.w / 2.;
                let flipped_x = (self.relative_bbox.x - center_offset) * -1. + center_offset;
                let mut flipped_relative_bbox = self.relative_bbox;
                flipped_relative_bbox.x = flipped_x;
                return flipped_relative_bbox.offset(self.pos);
            }
        }
        self.relative_bbox.offset(self.pos)
    }

    pub fn at_bottom_left(mut self, rect: &Rect) -> Self {
        self.pos.x = rect.left() - self.relative_bbox.left();
        self.pos.y = rect.bottom() - self.relative_bbox.bottom();
        self
    }

    pub fn at_top_left(mut self, rect: &Rect) -> Self {
        self.pos.x = rect.left() - self.relative_bbox.left();
        self.pos.y = rect.top() - self.relative_bbox.top();
        self
    }

    pub fn update_looping_frame_number(&mut self, time: &GameTime) {
        if let Some(sprite) = self.renderer {
            self.current_frame_number = time.looping_frame_number(&sprite);
        }
    }

    pub fn draw_current_frame(&self) {
        if let Some(sprite) = self.renderer {
            sprite.draw_ex(
                self.pos.x,
                self.pos.y,
                self.current_frame_number,
                SpriteDrawParams {
                    flip_x: self.is_facing_left,
                    ..Default::default()
                },
            );
        }
    }

    pub fn draw_debug_rects(&self) {
        if let Some(sprite) = self.renderer {
            sprite.draw_debug_rect(self.pos.x, self.pos.y, GREEN);
        }
        draw_rect_lines(&self.bbox(), 2., PURPLE);
    }
}
