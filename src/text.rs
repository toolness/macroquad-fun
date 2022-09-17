use macroquad::{
    prelude::{Rect, WHITE},
    text::draw_text,
};

use crate::{level::Level, sprite_component::SpriteComponent};

pub fn draw_level_text(sprite: &SpriteComponent, level: &Level, camera_rect: &Rect) {
    if let Some(text) = level.get_text(&sprite.bbox()) {
        let mut y = camera_rect.y + 128.;
        for line in text {
            draw_text(line, camera_rect.x + 32., y, 32.0, WHITE);
            y += 36.;
        }
    }
}
