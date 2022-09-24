use macroquad::prelude::{Rect, Vec2, BLUE};

use crate::{
    collision::Collider, dynamic_collider::DynamicColliderComponent, entity::Entity,
    physics::PhysicsComponent, sprite_component::SpriteComponent,
};

pub fn create_moving_platform(start_rect: Rect, _endpoint: Vec2) -> Entity {
    // TODO: Actually use endpoint!
    let relative_bbox = start_rect.offset(-start_rect.point());
    return Entity {
        sprite: SpriteComponent {
            pos: start_rect.point(),
            relative_bbox,
            color: Some(BLUE),
            ..Default::default()
        },
        physics: PhysicsComponent {
            defies_gravity: true,
            ..Default::default()
        },
        dynamic_collider: Some(DynamicColliderComponent::new(Collider {
            rect: relative_bbox,
            enable_top: true,
            enable_bottom: true,
            enable_left: true,
            enable_right: true,
            ..Default::default()
        })),
        ..Default::default()
    };
}
