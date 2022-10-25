use macroquad::prelude::WHITE;

use crate::{
    config::config, game_assets::game_assets, level::Level, sprite_component::SpriteComponent,
};

pub fn draw_level_text(sprite: &SpriteComponent, level: &Level) {
    if let Some(text) = level.get_text(&sprite.bbox()) {
        let font = &game_assets().font;
        let mut y = 128.;
        let line_height = (font.char_height as f32 + 2.) * config().sprite_scale;
        for line in text {
            font.draw_text(line, 32., y, WHITE);
            y += line_height;
        }
    }
}
