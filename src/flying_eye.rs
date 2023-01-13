use macroquad::prelude::{Rect, Vec2};

use crate::{
    attachment::AttachableComponent,
    config::config,
    entity::{Entity, EntityMap},
    game_assets::game_assets,
    materials::replace_colors_with_image,
    physics::{PhysicsCollisionBehavior, PhysicsComponent},
    sprite_component::{LeftFacingRendering, SpriteComponent},
    steering::SteeringComponent,
    time::GameTime,
};

#[derive(Clone, Copy)]
pub struct FlyingEyeComponent();

pub fn create_flying_eye(start_rect: Rect, base_velocity: Vec2) -> Entity {
    let assets = &game_assets().flying_eye;
    Entity {
        sprite: SpriteComponent {
            base_relative_bbox: assets.flight_bbox,
            sprite: Some(&assets.flight),
            left_facing_rendering: LeftFacingRendering::FlipBoundingBox,
            material: replace_colors_with_image(&assets.color_replacements),
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
        steering: Some(SteeringComponent::default()),
        attachable: Some(AttachableComponent()),
        ..Default::default()
    }
}

pub fn flying_eye_movement_system(entities: &mut EntityMap, time: &GameTime) {
    for (_id, entity) in entities.iter_mut() {
        if entity.flying_eye.is_some() {
            entity.sprite.is_facing_left = entity.physics.velocity.x < 0.;
            entity.sprite.update_looping_frame_number(time);
        }
    }
}
