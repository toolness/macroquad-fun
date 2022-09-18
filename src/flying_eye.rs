use macroquad::prelude::{Rect, Vec2};

use crate::{
    attachment::AttachableComponent,
    config::config,
    entity::{Entity, EntityMap},
    game_sprites::game_sprites,
    physics::{PhysicsCollisionBehavior, PhysicsComponent},
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
        physics: PhysicsComponent {
            velocity: base_velocity * config().flying_eye_speed,
            defies_gravity: true,
            collision_behavior: PhysicsCollisionBehavior::ReverseDirectionXY,
            ..Default::default()
        },
        flying_eye: Some(FlyingEyeComponent()),
        attachable: Some(AttachableComponent()),
        ..Default::default()
    }
}

const CARRY_Y_OFFSET: f32 = 10.0;

pub fn carry_entity(carrier: &SpriteComponent, passenger: &mut SpriteComponent) {
    let config = config();
    let bbox = carrier.bbox();
    let passenger_bbox = passenger.bbox();
    let y_diff = bbox.bottom() - config.sprite_scale * CARRY_Y_OFFSET - passenger_bbox.top();
    let x_diff = bbox.left() - passenger_bbox.left();
    passenger.pos += Vec2::new(x_diff, y_diff);
    passenger.is_facing_left = carrier.is_facing_left;
}

pub fn flying_eye_movement_system(entities: &mut EntityMap, time: &GameTime) {
    for entity in entities.values_mut() {
        if entity.flying_eye.is_some() {
            entity.sprite.is_facing_left = entity.physics.velocity.x < 0.;
            entity.sprite.update_looping_frame_number(time);
        }
    }
}
