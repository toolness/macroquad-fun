use macroquad::{
    prelude::{Rect, Vec2, BLUE, YELLOW},
    shapes::draw_line,
};

use crate::{
    config::config,
    dynamic_collider::{DynamicColliderComponent, RelativeCollider},
    entity::{Entity, EntityMap},
    physics::PhysicsComponent,
    sprite_component::SpriteComponent,
};

pub struct MovingPlatformComponent {
    start_point: Vec2,
    end_point: Vec2,
    is_moving_towards_start: bool,
}

impl MovingPlatformComponent {
    fn target(&self) -> Vec2 {
        if self.is_moving_towards_start {
            self.start_point
        } else {
            self.end_point
        }
    }
}

pub fn moving_platform_system(entities: &mut EntityMap) {
    let config = config();
    for entity in entities.values_mut() {
        if let Some(moving_platform) = entity.moving_platform.as_mut() {
            let target = moving_platform.target();
            let direction_to_target = target - entity.sprite.pos;
            if entity.physics.velocity == Vec2::ZERO {
                entity.physics.velocity =
                    direction_to_target.normalize() * config.moving_platform_speed;
            } else {
                let is_moving_towards_target =
                    entity.physics.velocity.dot(direction_to_target) > 0.;
                if !is_moving_towards_target {
                    entity.sprite.pos = target;
                    entity.physics.velocity = Vec2::ZERO;
                    moving_platform.is_moving_towards_start =
                        !moving_platform.is_moving_towards_start;
                }
            }
        }
    }
}

pub fn draw_moving_platform_debug_targets(entities: &EntityMap) {
    for entity in entities.values() {
        if let Some(moving_platform) = &entity.moving_platform {
            let target = moving_platform.target();
            draw_line(
                entity.sprite.pos.x,
                entity.sprite.pos.y,
                target.x,
                target.y,
                1.,
                YELLOW,
            );
        }
    }
}

pub fn create_moving_platform(start_rect: Rect, end_point: Vec2) -> Entity {
    let start_point = start_rect.point();
    let relative_bbox = start_rect.offset(-start_point);
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
        dynamic_collider: Some(DynamicColliderComponent::new(RelativeCollider {
            rect: relative_bbox,
            enable_top: true,
            enable_bottom: true,
            enable_left: true,
            enable_right: true,
        })),
        moving_platform: Some(MovingPlatformComponent {
            start_point,
            end_point,
            is_moving_towards_start: false,
        }),
        ..Default::default()
    };
}
