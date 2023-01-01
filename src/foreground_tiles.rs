use macroquad::prelude::Rect;

use crate::{
    entity::Entity,
    physics::PhysicsComponent,
    sprite_component::{Renderer, SpriteComponent},
    z_index::ZIndexComponent,
};

pub fn create_foreground_tiles(start_rect: Rect) -> Entity {
    return Entity {
        sprite: SpriteComponent {
            renderer: Renderer::EntityTiles(start_rect),
            ..Default::default()
        }
        .with_pos_and_size(&start_rect),
        physics: PhysicsComponent {
            defies_gravity: true,
            ..Default::default()
        },
        z_index: ZIndexComponent::new(600),
        ..Default::default()
    };
}
