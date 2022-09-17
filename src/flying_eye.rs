use macroquad::prelude::{Rect, Vec2};

use crate::{
    collision::{collision_resolution_loop, process_collision},
    config::config,
    entity::Entity,
    game_sprites::game_sprites,
    level::Level,
    math_util::are_opposites,
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

fn maybe_reverse_direction_xy(entity: &mut Entity, displacement: &Vec2) {
    if are_opposites(displacement.x, entity.velocity.x) {
        entity.velocity.x = -entity.velocity.x;
    }
    if are_opposites(displacement.y, entity.velocity.y) {
        entity.velocity.y = -entity.velocity.y;
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

pub fn update_flying_eye(entity: &mut Entity, level: &Level, time: &GameTime) -> Option<()> {
    entity.flying_eye.as_ref()?;
    let prev_bbox = entity.sprite.bbox();
    entity.sprite.pos += entity.velocity * time.time_since_last_frame as f32;

    collision_resolution_loop(|| {
        let bbox = entity.sprite.bbox();

        for collider in level
            .iter_colliders(&bbox)
            .chain(level.iter_bounds_as_colliders())
        {
            if let Some(collision) = process_collision(&collider, &prev_bbox, &bbox) {
                if collision.displacement != Vec2::ZERO {
                    entity.sprite.pos += collision.displacement;
                    maybe_reverse_direction_xy(entity, &collision.displacement);
                    return true;
                }
            }
        }
        false
    });

    entity.sprite.is_facing_left = entity.velocity.x < 0.;
    entity.sprite.update_looping_frame_number(time);
    Some(())
}
