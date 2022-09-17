use macroquad::prelude::{Rect, Vec2};

use crate::{
    animator::Animator,
    collision::{collision_resolution_loop, process_collision, Side},
    config::config,
    entity::Entity,
    game_sprites::game_sprites,
    level::Level,
    math_util::are_opposites,
    player::Player,
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
        ..Default::default()
    }
}

fn maybe_reverse_direction_x(velocity: &mut Vec2, displacement: &Vec2) {
    if are_opposites(displacement.x, velocity.x) {
        velocity.x = -velocity.x;
    }
}

pub fn update_mushroom(
    entity: &mut Entity,
    player: &Player,
    level: &Level,
    time: &GameTime,
) -> Option<()> {
    let mushroom = entity.mushroom.as_mut()?;
    let velocity = &mut entity.velocity;
    let sprite = &mut entity.sprite;

    match &mushroom.state {
        MushroomState::Dead => {
            if player.sprite_component().bbox().overlaps(&sprite.bbox()) {
                mushroom.state = MushroomState::Rezzing(Animator::new(dead_frame(), true, &time));
            }
        }
        MushroomState::Rezzing(animator) => {
            if animator.is_done(&time) {
                mushroom.state = MushroomState::Alive;
                sprite.renderer = Some(&game_sprites().mushroom.run);
                velocity.x = config().mushroom_speed;
            }
        }
        MushroomState::Alive => {
            velocity.y += config().gravity * time.time_since_last_frame as f32;
            let prev_bbox = sprite.bbox();
            sprite.pos += *velocity * time.time_since_last_frame as f32;

            collision_resolution_loop(|| {
                let bbox = sprite.bbox();

                for collider in level
                    .iter_colliders(&bbox)
                    .chain(level.iter_bounds_as_colliders())
                {
                    if let Some(collision) = process_collision(&collider, &prev_bbox, &bbox) {
                        if collision.side == Side::Top {
                            velocity.y = 0.;
                        }
                        if collision.displacement != Vec2::ZERO {
                            sprite.pos += collision.displacement;
                            maybe_reverse_direction_x(velocity, &collision.displacement);
                            return true;
                        }
                    }
                }
                false
            });
            entity.sprite.is_facing_left = entity.velocity.x < 0.;
        }
    }
    mushroom.set_current_frame_number(time, &mut entity.sprite);
    Some(())
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
