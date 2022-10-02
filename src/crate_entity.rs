use macroquad::prelude::{Rect, BROWN};

use crate::{
    dynamic_collider::{DynamicColliderComponent, RelativeCollider},
    entity::Entity,
    physics::{PhysicsCollisionBehavior, PhysicsComponent},
    push::PushComponent,
    sprite_component::{Renderer, SpriteComponent},
};

pub fn create_crate(start_rect: Rect) -> Entity {
    let start_point = start_rect.point();
    let relative_bbox = start_rect.offset(-start_point);
    Entity {
        sprite: SpriteComponent {
            pos: start_point,
            relative_bbox,
            renderer: Renderer::Rectangle,
            color: Some(BROWN),
            ..Default::default()
        },
        physics: PhysicsComponent {
            collision_behavior: PhysicsCollisionBehavior::Stop,
            ..Default::default()
        },
        dynamic_collider: Some(DynamicColliderComponent::new(RelativeCollider {
            rect: relative_bbox,
            enable_top: true,
            enable_bottom: true,
            enable_left: true,
            enable_right: true,
        })),
        push: Some(PushComponent {
            pushable_coefficient: 0.1,
            ..Default::default()
        }),
        ..Default::default()
    }
}
