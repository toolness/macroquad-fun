use macroquad::prelude::{Rect, Vec2};

use crate::{
    animator::Animator,
    collision::{collision_resolution_loop, process_collision},
    config::config,
    game_sprites::game_sprites,
    level::Level,
    player::Player,
    sprite_entity::SpriteEntity,
    time::GameTime,
};

enum MushroomState {
    Dead,
    Rezzing(Animator),
    Alive,
}

pub struct Mushroom {
    id: u64,
    entity: SpriteEntity,
    state: MushroomState,
    dead_frame: u32,
    velocity: Vec2,
}

impl Mushroom {
    pub fn new(id: u64, start_rect: Rect) -> Self {
        let sprites = &game_sprites().mushroom;
        let relative_bbox = sprites.idle_bbox;
        let death_sprite = &sprites.death;
        let entity = SpriteEntity {
            pos: Vec2::new(
                start_rect.left() - relative_bbox.x,
                start_rect.bottom() - relative_bbox.bottom(),
            ),
            relative_bbox,
            sprite: Some(&death_sprite),
            flip_bbox_when_facing_left: true,
            ..Default::default()
        };
        Mushroom {
            id,
            entity,
            state: MushroomState::Dead,
            dead_frame: death_sprite.last_frame(),
            velocity: Vec2::new(0., 0.),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn draw(&self, time: &GameTime) {
        match &self.state {
            MushroomState::Dead => {
                self.entity.draw_frame(self.dead_frame);
            }
            MushroomState::Rezzing(animator) => {
                self.entity.draw_frame(animator.get_frame(&time));
            }
            MushroomState::Alive => {
                self.entity.draw(&time);
            }
        }
    }

    fn maybe_reverse_direction(&mut self, displacement: &Vec2) {
        if displacement.x > 0. && self.velocity.x < 0.
            || displacement.x < 0. && self.velocity.x > 0.
        {
            self.velocity.x = -self.velocity.x;
        }
    }

    pub fn update(&mut self, player: &Player, level: &Level, time: &GameTime) {
        match &self.state {
            MushroomState::Dead => {
                if player.entity().bbox().overlaps(&self.entity.bbox()) {
                    self.state =
                        MushroomState::Rezzing(Animator::new(self.dead_frame, true, &time));
                }
            }
            MushroomState::Rezzing(animator) => {
                if animator.is_done(&time) {
                    self.state = MushroomState::Alive;
                    self.entity.sprite = Some(&game_sprites().mushroom.run);
                    self.velocity.x = config().mushroom_speed
                }
            }
            MushroomState::Alive => {
                let prev_bbox = self.entity.bbox();
                self.entity.pos += self.velocity * time.time_since_last_frame as f32;

                collision_resolution_loop(|| {
                    let bbox = self.entity.bbox();

                    for collider in level
                        .iter_colliders(&bbox)
                        .chain(level.iter_bounds_as_colliders())
                    {
                        if let Some(collision) = process_collision(&collider, &prev_bbox, &bbox) {
                            if collision.displacement != Vec2::ZERO {
                                self.entity.pos += collision.displacement;
                                self.maybe_reverse_direction(&collision.displacement);
                                return true;
                            }
                        }
                    }
                    false
                });
                self.entity.is_facing_left = self.velocity.x < 0.;
            }
        }
    }

    pub fn entity(&self) -> &SpriteEntity {
        &self.entity
    }
}
