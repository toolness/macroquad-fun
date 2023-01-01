use macroquad::window::{screen_height, screen_width};

use crate::{
    config::config,
    entity::EntityMap,
    game_assets::game_assets,
    level::{EntityKind, Level},
};

use std::fmt::Write;

fn count_total_and_remaining_gems(entities: &EntityMap, level: &Level) -> (u32, u32) {
    let mut total = 0;
    let mut remaining = 0;

    for entity in level.entities.values() {
        if matches!(entity.kind, EntityKind::Gem) {
            if entities.get_id_for_iid(entity.iid).is_some() {
                remaining += 1;
            }
            total += 1;
        }
    }

    (total, remaining)
}

pub fn draw_gem_counter(entities: &EntityMap, level: &Level) {
    let (total, remaining) = count_total_and_remaining_gems(entities, level);

    if total == remaining {
        // Don't show the gem counter if there are no gems to collect.
        //
        // But also, don't show it if the player has no gems: they might
        // not yet have learned what gems are and we don't want to overload
        // them with information.
        return;
    }

    let mut string: heapless::String<100> = heapless::String::new();
    let font = &game_assets().font;

    if remaining == 0 {
        write!(string, "You have collected all the gems!").unwrap();
    } else if remaining == 1 {
        write!(string, "One gem remains.").unwrap();
    } else {
        write!(string, "{} gems remain.", remaining).unwrap();
    }

    let scale = config().sprite_scale;
    let line_height = (font.char_height as f32) * scale;
    let string_width = (font.char_width as f32) * scale * string.len() as f32;

    font.draw_text(
        &string,
        screen_width() - string_width - 32.,
        screen_height() - line_height - 32.,
        macroquad::prelude::WHITE,
    );
}
