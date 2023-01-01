use macroquad::prelude::{Rect, PINK};

use crate::{
    collision::CollisionFlags,
    config::config,
    dynamic_collider::{DynamicColliderComponent, RelativeCollider},
    entity::Entity,
    level::{MovingPlatformArgs, RendererType},
    physics::PhysicsComponent,
    route::RouteComponent,
    sprite_component::{Renderer, SpriteComponent},
};

pub fn create_moving_platform(start_rect: Rect, args: &MovingPlatformArgs) -> Entity {
    let mut sprite = SpriteComponent::default().with_pos_and_size(&start_rect);
    match args.renderer_type {
        RendererType::EntityTiles => {
            sprite.renderer = Renderer::EntityTiles(start_rect);
        }
        RendererType::SolidRectangle => {
            // Used only for prototyping.
            sprite.renderer = Renderer::SolidRectangle(sprite.relative_bbox());
            sprite.color = Some(PINK);
        }
    };

    return Entity {
        sprite,
        physics: PhysicsComponent {
            defies_gravity: true,
            ..Default::default()
        },
        dynamic_collider: Some(DynamicColliderComponent::new(RelativeCollider {
            rect: sprite.relative_bbox(),
            collision_flags: CollisionFlags::ENVIRONMENT,
            enable_top: true,
            enable_bottom: true,
            enable_left: true,
            enable_right: true,
        })),
        route: Some(RouteComponent {
            start_point: sprite.pos,
            end_point: args.end_point,
            stop_when_blocked: args.stop_when_blocked,
            is_moving: args.ping_pong,
            ping_pong: args.ping_pong,
            is_moving_towards_start: false,
            speed: config().moving_platform_speed,
        }),
        ..Default::default()
    };
}
