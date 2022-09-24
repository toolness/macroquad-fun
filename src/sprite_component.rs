use macroquad::{
    prelude::{Color, Rect, Vec2, GREEN, PURPLE, WHITE},
    shapes::draw_rectangle,
};

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
    pub color: Option<Color>,
    pub is_facing_left: bool,
    pub flip_bbox_when_facing_left: bool,
    pub current_frame_number: u32,
}

impl SpriteComponent {
    pub fn calculate_absolute_bounding_box(&self, relative_bbox: &Rect) -> Rect {
        if self.flip_bbox_when_facing_left && self.is_facing_left {
            if let Some(sprite) = self.renderer {
                let center_offset = sprite.frame_width() / 2. - relative_bbox.w / 2.;
                let flipped_x = (self.relative_bbox.x - center_offset) * -1. + center_offset;
                let mut flipped_relative_bbox = *relative_bbox;
                flipped_relative_bbox.x = flipped_x;
                return flipped_relative_bbox.offset(self.pos);
            }
        }
        relative_bbox.offset(self.pos)
    }

    pub fn bbox(&self) -> Rect {
        self.calculate_absolute_bounding_box(&self.relative_bbox)
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
        let color = self.color.unwrap_or(WHITE);
        if let Some(sprite) = self.renderer {
            sprite.draw_ex(
                self.pos.x,
                self.pos.y,
                self.current_frame_number,
                SpriteDrawParams {
                    flip_x: self.is_facing_left,
                    color,
                    ..Default::default()
                },
            );
        } else {
            draw_rectangle(
                self.pos.x,
                self.pos.y,
                self.relative_bbox.w,
                self.relative_bbox.h,
                color,
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
