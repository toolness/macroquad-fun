use macroquad::prelude::{Rect, Vec2, GREEN, PURPLE};

use crate::{drawing::draw_rect_lines, game_sprites::game_sprites, sprite::Sprite};

pub struct FlyingEye {
    pos: Vec2,
    relative_bbox: Rect,
}

impl FlyingEye {
    pub fn new(start_rect: Rect) -> Self {
        let relative_bbox = game_sprites().flying_eye.flight_bbox;
        FlyingEye {
            pos: Vec2::new(
                start_rect.left() - relative_bbox.x,
                start_rect.top() - relative_bbox.y,
            ),
            relative_bbox,
        }
    }

    pub fn bbox(&self) -> Rect {
        self.relative_bbox.offset(self.pos)
    }

    fn sprite<'a>(&self) -> &'static Sprite {
        &game_sprites().flying_eye.flight
    }

    pub fn draw(&self, absolute_frame_number: u32) {
        let sprite = self.sprite();
        sprite.draw(
            self.pos.x,
            self.pos.y,
            absolute_frame_number % sprite.num_frames(),
        );
    }

    pub fn draw_debug_rects(&self) {
        let sprite = self.sprite();

        sprite.draw_debug_rect(self.pos.x, self.pos.y, GREEN);
        draw_rect_lines(&self.bbox(), 2., PURPLE);
    }
}
