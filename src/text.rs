use macroquad::prelude::{Rect, WHITE};

use crate::{
    config::config, game_assets::game_assets, level::Level, sprite_component::SpriteComponent,
};

pub fn draw_level_text(sprite: &SpriteComponent, level: &Level, camera_rect: &Rect) {
    if let Some(text) = level.get_text(&sprite.bbox()) {
        let font = &game_assets().font;
        let mut y = camera_rect.y + 128.;
        let line_height = (font.char_height as f32 + 2.) * config().sprite_scale;
        for line in text {
            font.draw_text(line, camera_rect.x + 32., y, WHITE);
            y += line_height;
        }
    }
}
