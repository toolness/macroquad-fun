use macroquad::prelude::{Rect, Vec2, GREEN, PURPLE};

use crate::{
    drawing::draw_rect_lines,
    sprite::{Sprite, SpriteDrawParams},
    time::GameTime,
};

#[derive(Default)]
pub struct SpriteEntity {
    pub pos: Vec2,
    pub relative_bbox: Rect,
    pub sprite: Option<&'static Sprite>,
    pub is_facing_left: bool,
    pub flip_bbox_when_facing_left: bool,
}

impl SpriteEntity {
    pub fn bbox(&self) -> Rect {
        if self.flip_bbox_when_facing_left && self.is_facing_left {
            if let Some(sprite) = self.sprite {
                let center_offset = sprite.frame_width() / 2. - self.relative_bbox.w / 2.;
                let flipped_x = (self.relative_bbox.x - center_offset) * -1. + center_offset;
                let mut flipped_relative_bbox = self.relative_bbox;
                flipped_relative_bbox.x = flipped_x;
                return flipped_relative_bbox.offset(self.pos);
            }
        }
        self.relative_bbox.offset(self.pos)
    }

    pub fn position_at_bottom_left(&mut self, rect: &Rect) {
        self.pos.x = rect.left() - self.relative_bbox.left();
        self.pos.y = rect.bottom() - self.relative_bbox.bottom();
    }

    pub fn position_at_top_left(&mut self, rect: &Rect) {
        self.pos.x = rect.left() - self.relative_bbox.left();
        self.pos.y = rect.top() - self.relative_bbox.top();
    }

    pub fn draw(&self, time: &GameTime) {
        if let Some(sprite) = self.sprite {
            self.draw_frame(time.looping_frame_number(&sprite));
        }
    }

    pub fn draw_frame(&self, frame_number: u32) {
        if let Some(sprite) = self.sprite {
            sprite.draw_ex(
                self.pos.x,
                self.pos.y,
                frame_number,
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
