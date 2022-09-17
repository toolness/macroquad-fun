use macroquad::{
    prelude::{Rect, WHITE},
    text::draw_text,
};

use crate::{level::Level, player::Player};

pub fn draw_level_text(player: &Player, level: &Level, camera_rect: &Rect) {
    if let Some(text) = level.get_text(&player.sprite_component().bbox()) {
        let mut y = camera_rect.y + 128.;
        for line in text {
            draw_text(line, camera_rect.x + 32., y, 32.0, WHITE);
            y += 36.;
        }
    }
}
