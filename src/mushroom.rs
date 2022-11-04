use macroquad::prelude::{Rect, Vec2};

use crate::{
    animator::Animator,
    config::config,
    dynamic_collider::{DynamicColliderComponent, RelativeCollider},
    entity::{Entity, EntityMap},
    game_assets::game_assets,
    physics::{PhysicsCollisionBehavior, PhysicsComponent},
    sprite_component::{LeftFacingRendering, Renderer, SpriteComponent},
    time::GameTime,
};

#[derive(Clone, Copy)]
pub struct MushroomComponent {
    state: MushroomState,
}

#[derive(Clone, Copy)]
pub enum MushroomState {
    Dead,
    Rezzing(Animator),
    Alive,
}

fn dead_frame() -> u32 {
    game_assets().mushroom.death.last_frame()
}

pub fn create_mushrom(start_rect: Rect) -> Entity {
    let sprites = &game_assets().mushroom;
    let death_sprite = &sprites.death;
    Entity {
        sprite: SpriteComponent {
            relative_bbox: sprites.idle_bbox,
            renderer: Renderer::Sprite(&death_sprite),
            left_facing_rendering: LeftFacingRendering::FlipBoundingBox,
            ..Default::default()
        }
        .at_bottom_left(&start_rect),
        mushroom: Some(MushroomComponent {
            state: MushroomState::Dead,
        }),
        physics: PhysicsComponent {
            collision_behavior: PhysicsCollisionBehavior::ReverseDirectionX,
            gravity_coefficient: Some(0.5),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn mushroom_movement_system(entities: &mut EntityMap, time: &GameTime) {
    let player_bbox = entities.player().sprite.bbox();
    for (_id, entity) in entities.iter_mut() {
        if let Some(mushroom) = entity.mushroom.as_mut() {
            let velocity = &mut entity.physics.velocity;
            let sprite = &mut entity.sprite;
            let dynamic_collider = &mut entity.dynamic_collider;
            update_mushroom(
                mushroom,
                velocity,
                sprite,
                dynamic_collider,
                &player_bbox,
                time,
            );
        }
    }
}

fn update_mushroom(
    mushroom: &mut MushroomComponent,
    velocity: &mut Vec2,
    sprite: &mut SpriteComponent,
    dynamic_collider: &mut Option<DynamicColliderComponent>,
    player_bbox: &Rect,
    time: &GameTime,
) {
    match &mushroom.state {
        MushroomState::Dead => {
            if player_bbox.overlaps(&sprite.bbox()) {
                mushroom.state = MushroomState::Rezzing(Animator::new(dead_frame(), true, &time));
            }
        }
        MushroomState::Rezzing(animator) => {
            if animator.is_done(&time) {
                mushroom.state = MushroomState::Alive;
                sprite.renderer = Renderer::Sprite(&game_assets().mushroom.run);
                velocity.x = config().mushroom_speed;
                let _ = dynamic_collider.insert(DynamicColliderComponent::new(RelativeCollider {
                    rect: game_assets().mushroom.platform_bbox,
                    enable_top: true,
                    ..Default::default()
                }));
            }
        }
        MushroomState::Alive => {
            sprite.is_facing_left = velocity.x < 0.;
        }
    }
    mushroom.set_current_frame_number(time, sprite);
}

impl MushroomComponent {
    fn set_current_frame_number(&self, time: &GameTime, sprite: &mut SpriteComponent) {
        match &self.state {
            MushroomState::Dead => {
                sprite.current_frame_number = dead_frame();
            }
            MushroomState::Rezzing(animator) => {
                sprite.current_frame_number = animator.get_frame(&time);
            }
            MushroomState::Alive => {
                sprite.update_looping_frame_number(&time);
            }
        }
    }
}
