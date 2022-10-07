use macroquad::prelude::{Rect, BROWN};

use crate::{
    collision::CollisionFlags,
    config::config,
    dynamic_collider::{DynamicColliderComponent, RelativeCollider},
    entity::Entity,
    math_util::contract_rect,
    physics::{PhysicsCollisionBehavior, PhysicsComponent},
    push::PushComponent,
    sprite_component::{Renderer, SpriteComponent},
    z_index::ZIndexComponent,
};

pub fn create_crate(start_rect: Rect) -> Entity {
    let start_point = start_rect.point();
    let relative_bbox = start_rect.offset(-start_point);
    Entity {
        sprite: SpriteComponent {
            pos: start_point,
            relative_bbox: contract_rect(&relative_bbox, config().sprite_scale * 0.5),
            renderer: Renderer::SolidRectangle(relative_bbox),
            color: Some(BROWN),
            ..Default::default()
        },
        physics: PhysicsComponent {
            collision_behavior: PhysicsCollisionBehavior::Stop,
            collision_flags: CollisionFlags::PLAYER_ONLY | CollisionFlags::ENVIRONMENT,
            ..Default::default()
        },
        z_index: ZIndexComponent::new(100),
        dynamic_collider: Some(DynamicColliderComponent::new(RelativeCollider {
            rect: relative_bbox,
            collision_flags: CollisionFlags::PLAYER_ONLY,
            enable_top: true,
            enable_bottom: true,
            enable_left: true,
            enable_right: true,
        })),
        push: Some(PushComponent {
            pushable_coefficient: config().crate_pushable_coefficient,
            ..Default::default()
        }),
        ..Default::default()
    }
}
