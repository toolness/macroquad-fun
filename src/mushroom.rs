use macroquad::prelude::{Rect, Vec2};

use crate::{
    animator::Animator,
    config::config,
    dynamic_platform::DynamicPlatformComponent,
    entity::{Entity, EntityMap, EntityMapHelpers},
    game_sprites::game_sprites,
    physics::{PhysicsCollisionBehavior, PhysicsComponent},
    sprite_component::SpriteComponent,
    time::GameTime,
};

pub struct MushroomComponent {
    state: MushroomState,
}

pub enum MushroomState {
    Dead,
    Rezzing(Animator),
    Alive,
}

fn dead_frame() -> u32 {
    game_sprites().mushroom.death.last_frame()
}

pub fn create_mushrom(start_rect: Rect) -> Entity {
    let sprites = &game_sprites().mushroom;
    let death_sprite = &sprites.death;
    Entity {
        sprite: SpriteComponent {
            relative_bbox: sprites.idle_bbox,
            renderer: Some(&death_sprite),
            flip_bbox_when_facing_left: true,
            ..Default::default()
        }
        .at_bottom_left(&start_rect),
        mushroom: Some(MushroomComponent {
            state: MushroomState::Dead,
        }),
        physics: PhysicsComponent {
            collision_behavior: PhysicsCollisionBehavior::ReverseDirectionX,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn mushroom_movement_system(entities: &mut EntityMap, time: &GameTime) {
    let player_bbox = entities.player().sprite.bbox();
    for entity in entities.values_mut() {
        if let Some(mushroom) = entity.mushroom.as_mut() {
            let velocity = &mut entity.physics.velocity;
            let sprite = &mut entity.sprite;
            let dynamic_platform = &mut entity.dynamic_platform;
            update_mushroom(
                mushroom,
                velocity,
                sprite,
                dynamic_platform,
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
    dynamic_platform: &mut Option<DynamicPlatformComponent>,
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
                sprite.renderer = Some(&game_sprites().mushroom.run);
                velocity.x = config().mushroom_speed;
                let _ = dynamic_platform.insert(DynamicPlatformComponent {
                    bbox: game_sprites().mushroom.platform_bbox,
                });
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
