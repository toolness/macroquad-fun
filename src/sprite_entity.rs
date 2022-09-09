use macroquad::prelude::{Rect, Vec2, GREEN, PURPLE};

use crate::{drawing::draw_rect_lines, sprite::Sprite};

pub struct SpriteEntity {
    pub pos: Vec2,
    pub relative_bbox: Rect,
    pub sprite: &'static Sprite,
}

impl SpriteEntity {
    pub fn bbox(&self) -> Rect {
        self.relative_bbox.offset(self.pos)
    }

    pub fn draw(&self, absolute_frame_number: u32) {
        self.sprite.draw(
            self.pos.x,
            self.pos.y,
            absolute_frame_number % self.sprite.num_frames(),
        );
    }

    pub fn draw_debug_rects(&self) {
        self.sprite.draw_debug_rect(self.pos.x, self.pos.y, GREEN);
        draw_rect_lines(&self.bbox(), 2., PURPLE);
    }
}
