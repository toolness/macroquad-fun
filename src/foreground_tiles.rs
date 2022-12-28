use macroquad::prelude::Rect;

use crate::{
    entity::Entity,
    physics::PhysicsComponent,
    sprite_component::{Renderer, SpriteComponent},
    z_index::ZIndexComponent,
};

pub fn create_foreground_tiles(start_rect: Rect) -> Entity {
    let start_point = start_rect.point();
    let relative_bbox = start_rect.offset(-start_point);
    return Entity {
        sprite: SpriteComponent {
            pos: start_point,
            base_relative_bbox: relative_bbox,
            renderer: Renderer::EntityTiles(start_rect),
            ..Default::default()
        },
        physics: PhysicsComponent {
            defies_gravity: true,
            ..Default::default()
        },
        z_index: ZIndexComponent::new(600),
        ..Default::default()
    };
}
