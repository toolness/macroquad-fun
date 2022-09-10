use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode, Rect, Vec2};

use crate::{
    collision::{process_collision, Side},
    config::config,
    game_sprites::game_sprites,
    level::Level,
    running::RunManager,
    sprite::Sprite,
    sprite_entity::SpriteEntity,
    world::world,
};

pub struct Player {
    entity: SpriteEntity,
    is_in_air: bool,
    velocity: Vec2,
    x_impulse: f32,
    run_manager: RunManager,
}

const MAX_DISPLACEMENTS_PER_FRAME: u32 = 30;

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
        let mut displacements_this_frame = 0;

        loop {
            let bbox = self.entity.bbox();
            let mut displacement_occurred = false;
            for collider in level.iter_colliders(&bbox) {
                if let Some(collision) = process_collision(&collider, &prev_bbox, &bbox) {
                    match collision.side {
                        Side::Top => {
                            is_on_any_surface_this_frame = true;
                            self.velocity = Vec2::new(0., 0.);
                        }
                        Side::Bottom => {
                            self.velocity.y = 0.;
                        }
                        Side::Left | Side::Right => {
                            self.velocity.x = 0.;
                        }
                    }

                    if collision.displacement != Vec2::ZERO {
                        self.entity.pos += collision.displacement;
                        displacement_occurred = true;
                        break;
                    }
                }
            }
            if !displacement_occurred {
                break;
            }
            displacements_this_frame += 1;
            if displacements_this_frame > MAX_DISPLACEMENTS_PER_FRAME {
                println!(
                    "WARNING: stuck in possible displacement loop, aborting collision resolution."
                );
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

    pub fn maybe_switch_levels(&mut self, level: &'static Level) -> Option<&'static Level> {
        let world = world();
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
