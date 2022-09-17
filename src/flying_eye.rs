use std::collections::HashMap;

use macroquad::prelude::{Rect, Vec2};

use crate::{
    collision::{collision_resolution_loop, maybe_reverse_direction_xy, process_collision},
    config::config,
    entity::Entity,
    game_sprites::game_sprites,
    level::Level,
    sprite_component::SpriteComponent,
    time::GameTime,
};

pub struct FlyingEyeComponent();

pub fn create_flying_eye(start_rect: Rect, base_velocity: Vec2) -> Entity {
    Entity {
        sprite: SpriteComponent {
            relative_bbox: game_sprites().flying_eye.flight_bbox,
            renderer: Some(&game_sprites().flying_eye.flight),
            flip_bbox_when_facing_left: true,
            ..Default::default()
        }
        .at_top_left(&start_rect),
        velocity: base_velocity * config().flying_eye_speed,
        flying_eye: Some(FlyingEyeComponent()),
        ..Default::default()
    }
}

const CARRY_Y_OFFSET: f32 = 10.0;

pub fn carry_entity(carrier: &Entity, passenger: &mut SpriteComponent) {
    let config = config();
    let bbox = carrier.sprite.bbox();
    let passenger_bbox = passenger.bbox();
    let y_diff = bbox.bottom() - config.sprite_scale * CARRY_Y_OFFSET - passenger_bbox.top();
    let x_diff = bbox.left() - passenger_bbox.left();
    passenger.pos += Vec2::new(x_diff, y_diff);
    passenger.is_facing_left = carrier.sprite.is_facing_left;
}

pub fn flying_eye_movement_system(
    entities: &mut HashMap<u64, Entity>,
    level: &Level,
    time: &GameTime,
) {
    for entity in entities.values_mut() {
        if let Some(_flying_eye) = entity.flying_eye.as_mut() {
            update_flying_eye(&mut entity.velocity, &mut entity.sprite, level, time);
        }
    }
}

fn update_flying_eye(
    velocity: &mut Vec2,
    sprite: &mut SpriteComponent,
    level: &Level,
    time: &GameTime,
) {
    let prev_bbox = sprite.bbox();
    sprite.pos += *velocity * time.time_since_last_frame as f32;

    collision_resolution_loop(|| {
        let bbox = sprite.bbox();

        for collider in level
            .iter_colliders(&bbox)
            .chain(level.iter_bounds_as_colliders())
        {
            if let Some(collision) = process_collision(&collider, &prev_bbox, &bbox) {
                if collision.displacement != Vec2::ZERO {
                    sprite.pos += collision.displacement;
                    maybe_reverse_direction_xy(velocity, &collision.displacement);
                    return true;
                }
            }
        }
        false
    });

    sprite.is_facing_left = velocity.x < 0.;
    sprite.update_looping_frame_number(time);
}
