use macroquad::prelude::Rect;

use crate::{
    entity::Entity,
    game_assets::game_assets,
    physics::PhysicsComponent,
    sprite_component::{Renderer, SpriteComponent},
};

pub fn create_spear(start_rect: Rect) -> Entity {
    let assets = &game_assets().spear;
    Entity {
        sprite: SpriteComponent {
            relative_bbox: assets.spear_move_bbox,
            renderer: Renderer::Sprite(&assets.spear_move),
            ..Default::default()
        }
        .at_bottom_left(&start_rect),
        physics: PhysicsComponent {
            defies_gravity: true,
            ..Default::default()
        },
        ..Default::default()
    }
}
