use macroquad::prelude::{Rect, WHITE};

use crate::{
    config::config,
    entity::{Entity, EntityMap},
    game_assets::game_assets,
    level::Level,
    physics::PhysicsComponent,
    sprite_component::{Renderer, SpriteComponent},
};

#[derive(Clone, Copy)]
pub struct TextComponent {}

pub fn create_text_entity(rect: Rect) -> Entity {
    let start_point = rect.point();
    let relative_bbox = rect.offset(-start_point);
    Entity {
        sprite: SpriteComponent {
            pos: start_point,
            base_relative_bbox: relative_bbox,
            // Rendering is actually done via `draw_level_text()`, so we don't
            // use the sprite renderer.
            renderer: Renderer::Invisible,
            ..Default::default()
        },
        physics: PhysicsComponent {
            defies_gravity: true,
            ..Default::default()
        },
        text: Some(TextComponent {}),
        ..Default::default()
    }
}

pub fn draw_level_text(entities: &EntityMap, level: &Level) {
    let player_bbox = entities.main_player().sprite.bbox();
    for (_, entity) in entities.iter() {
        let Some(_) = entity.text else {
            continue
        };
        if !entity.sprite.bbox().overlaps(&player_bbox) {
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
        for line in lines {
            font.draw_text(line, 32., y, WHITE);
            y += line_height;
        }
        break;
    }
}
