use macroquad::prelude::{Rect, WHITE};

use crate::{
    config::config,
    entity::{Entity, EntityMap},
    game_assets::game_assets,
    level::Level,
    physics::PhysicsComponent,
    sprite_component::{Renderer, SpriteComponent},
    time::GameTime,
};

#[derive(Clone, Copy, Default)]
pub struct TextComponent {
    max_chars: u16,
    last_max_chars_change_frame_number: u64,
}

const MAX_TEXT_CHARS: u16 = 5000;

pub fn create_text_entity(rect: Rect) -> Entity {
    Entity {
        sprite: SpriteComponent {
            // Rendering is actually done via `draw_level_text()`, so we don't
            // use the sprite renderer.
            renderer: Renderer::Invisible,
            ..Default::default()
        }
        .with_pos_and_size(&rect),
        physics: PhysicsComponent {
            defies_gravity: true,
            ..Default::default()
        },
        text: Some(TextComponent::default()),
        ..Default::default()
    }
}

pub fn update_level_text(entities: &mut EntityMap, time: &GameTime) {
    let absolute_frame_number = (time.now * 1000.0 / config().ms_per_text_char) as u64;
    let player_bbox = entities.main_player().sprite.bbox();
    for (_, entity) in entities.iter_mut() {
        let Some(text) = entity.text.as_mut() else {
            continue
        };
        if !entity.sprite.bbox().overlaps(&player_bbox) {
            text.last_max_chars_change_frame_number = 0;
            text.max_chars = 0;
            continue;
        }
        if text.max_chars >= MAX_TEXT_CHARS {
            continue;
        }
        if absolute_frame_number > text.last_max_chars_change_frame_number {
            text.last_max_chars_change_frame_number = absolute_frame_number;
            text.max_chars += 1;
        }
    }
}

pub fn draw_level_text(entities: &EntityMap, level: &Level) {
    for (_, entity) in entities.iter() {
        let Some(text) = entity.text else {
            continue
        };
        if text.max_chars == 0 {
            continue;
        }
        let Some(iid) = entity.iid else {
            println!("WARNING: Entity with text component has no iid!");
            continue
        };
        let Some(lines) = level.get_text(&iid) else {
            println!("WARNING: Entity with text component has no text!");
            continue
        };

        let font = &game_assets().font;
        let mut y = 128.;
        let line_height = (font.char_height as f32 + 2.) * config().sprite_scale;
        let mut chars_left = text.max_chars as usize;
        for line in lines {
            let end_index = if line.len() > chars_left {
                let prev_chars_left = chars_left;
                chars_left = 0;
                prev_chars_left as usize
            } else {
                chars_left -= line.len();
                line.len()
            };
            let chars = &line[0..end_index];
            font.draw_text(chars, 32., y, WHITE);
            y += line_height;
        }
        break;
    }
}
