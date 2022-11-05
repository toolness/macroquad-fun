use macroquad::prelude::{Rect, Vec2};

use crate::{
    attachment::AttachableComponent,
    config::config,
    entity::{Entity, EntityMap},
    game_assets::game_assets,
    physics::{PhysicsCollisionBehavior, PhysicsComponent},
    sprite_component::{LeftFacingRendering, Renderer, SpriteComponent},
    time::GameTime,
};

#[derive(Clone, Copy)]
pub struct FlyingEyeComponent();

pub fn create_flying_eye(start_rect: Rect, base_velocity: Vec2) -> Entity {
    Entity {
        sprite: SpriteComponent {
            relative_bbox: game_assets().flying_eye.flight_bbox,
            renderer: Renderer::Sprite(&game_assets().flying_eye.flight),
            left_facing_rendering: LeftFacingRendering::FlipBoundingBox,
            material: Some(game_assets().replace_color_material),
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

pub fn flying_eye_movement_system(entities: &mut EntityMap, time: &GameTime) {
    for (_id, entity) in entities.iter_mut() {
        if entity.flying_eye.is_some() {
            entity.sprite.is_facing_left = entity.physics.velocity.x < 0.;
            entity.sprite.update_looping_frame_number(time);
        }
    }
}
