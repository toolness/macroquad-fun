use macroquad::prelude::{Rect, Vec2};

use crate::{
    collision::CollisionFlags,
    config::config,
    dynamic_collider::{DynamicColliderComponent, RelativeCollider},
    entity::Entity,
    physics::PhysicsComponent,
    route::RouteComponent,
    sprite_component::{Renderer, SpriteComponent},
};

pub fn create_moving_platform(start_rect: Rect, end_point: Vec2, ping_pong: bool) -> Entity {
    let start_point = start_rect.point();
    let relative_bbox = start_rect.offset(-start_point);
    return Entity {
        sprite: SpriteComponent {
            pos: start_rect.point(),
            relative_bbox,
            renderer: Renderer::EntityTiles(start_rect),
            ..Default::default()
        },
        physics: PhysicsComponent {
            defies_gravity: true,
            ..Default::default()
        },
        dynamic_collider: Some(DynamicColliderComponent::new(RelativeCollider {
            rect: relative_bbox,
            collision_flags: CollisionFlags::ENVIRONMENT,
            enable_top: true,
            enable_bottom: true,
            enable_left: true,
            enable_right: true,
        })),
        route: Some(RouteComponent {
            start_point,
            end_point,
            can_be_blocked: true, // TODO CHANGE THIS
            is_moving: ping_pong,
            ping_pong,
            is_moving_towards_start: false,
            speed: config().moving_platform_speed,
        }),
        ..Default::default()
    };
}
