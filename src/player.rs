use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode, Rect, Vec2};

use crate::{
    collision::{process_collision, Actor},
    config::config,
    game_sprites::game_sprites,
    level::{Level, World},
    running::RunManager,
    sprite::Sprite,
    sprite_entity::SpriteEntity,
};

pub struct Player {
    entity: SpriteEntity,
    is_in_air: bool,
    velocity: Vec2,
    x_impulse: f32,
    run_manager: RunManager,
}

impl Player {
    pub fn new(start_rect: Rect) -> Self {
        let relative_bbox = game_sprites().huntress.idle_bbox;
        let entity = SpriteEntity {
            pos: Vec2::new(
                start_rect.left() - relative_bbox.x,
                start_rect.bottom() - relative_bbox.bottom(),
            ),
            relative_bbox,
            ..Default::default()
        };
        Player {
            entity,
            is_in_air: false,
            velocity: Vec2::new(0., 0.),
            x_impulse: 0.,
            run_manager: RunManager::new(),
        }
    }

    pub fn entity(&self) -> &SpriteEntity {
        &self.entity
    }

    pub fn process_input_and_physics(&mut self, level: &Level, time_since_last_frame: f64) {
        let config = config();
        self.run_manager.update(
            time_since_last_frame,
            is_key_down(KeyCode::A),
            is_key_down(KeyCode::D),
        );
        self.x_impulse = 0.;

        if self.is_in_air {
            if is_key_down(KeyCode::Space) && self.velocity.y < 0. {
                self.velocity.y -=
                    config.long_jump_keypress_extra_force * time_since_last_frame as f32;
            }
            self.velocity.y += config.gravity * time_since_last_frame as f32;
            if self.run_manager.is_running() {
                self.velocity.x = self.run_manager.run_speed();
            }
        } else {
            if is_key_pressed(KeyCode::Space) {
                self.velocity = Vec2::new(self.run_manager.run_speed(), -config.jump_velocity);
                self.is_in_air = true
            } else {
                self.x_impulse = self.run_manager.run_speed();
            }
        }

        let prev_bbox = self.entity.bbox();
        self.entity.pos.x += (self.velocity.x + self.x_impulse) * time_since_last_frame as f32;
        self.entity.pos.y += self.velocity.y * time_since_last_frame as f32;

        let mut is_on_any_surface_this_frame = false;

        loop {
            let player_actor = Actor {
                prev_bbox,
                bbox: self.entity.bbox(),
                velocity: self.velocity,
            };
            let mut displacement_occurred = false;
            for collider in level.iter_colliders(&player_actor.bbox) {
                if let Some(collision) = process_collision(&collider, &player_actor) {
                    if collision.is_on_surface {
                        is_on_any_surface_this_frame = true;
                    }
                    if let Some(new_velocity) = collision.new_velocity {
                        self.velocity = new_velocity;
                    }

                    if collision.displacement.x != 0. || collision.displacement.y != 0. {
                        self.entity.pos += collision.displacement;
                        displacement_occurred = true;
                        break;
                    }
                }
            }
            if !displacement_occurred {
                break;
            }
        }

        if is_on_any_surface_this_frame {
            // The player just landed (or remains on the ground).
            self.is_in_air = false;
        } else if !self.is_in_air {
            // The player just fell off a ledge.
            self.is_in_air = true;
        }

        if !self.is_in_air && self.x_impulse != 0. {
            self.entity.is_facing_left = self.x_impulse < 0.;
        }

        self.entity.sprite = Some(self.sprite());
    }

    fn sprite(&self) -> &'static Sprite {
        let sprites = game_sprites();
        if self.is_in_air {
            if self.velocity.y >= 0. {
                &sprites.huntress.fall
            } else {
                &sprites.huntress.jump
            }
        } else {
            if self.x_impulse != 0. {
                &sprites.huntress.run
            } else {
                &sprites.huntress.idle
            }
        }
    }

    pub fn maybe_switch_levels<'a>(
        &mut self,
        level: &'a Level,
        world: &'a World,
    ) -> Option<&'a Level> {
        if !level.contains_majority_of(&self.entity.bbox()) {
            let world_pos = level.to_world_coords(&self.entity.pos);
            if let Some((new_level, new_pos)) =
                world.find_level_containing_majority_of(&world_pos, &self.entity.relative_bbox)
            {
                self.entity.pos = new_pos;
                return Some(new_level);
            }
        }
        None
    }
}
