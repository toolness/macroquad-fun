use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode, Rect, Vec2, GREEN, PURPLE};

use crate::{
    collision::{process_collision, Actor},
    config::Config,
    drawing::draw_rect_lines,
    game_sprites::GameSprites,
    level::Level,
    running::RunManager,
    sprite::{Sprite, SpriteDrawParams},
};

pub struct Player {
    pos: Vec2,
    is_in_air: bool,
    velocity: Vec2,
    is_facing_left: bool,
    relative_bbox: Rect,
    x_impulse: f32,
    run_manager: RunManager,
}

impl Player {
    pub fn new(start_rect: Rect, sprites: &GameSprites) -> Self {
        let relative_bbox = sprites.huntress.idle_bbox;
        Player {
            pos: Vec2::new(
                start_rect.left() - relative_bbox.x,
                start_rect.bottom() - relative_bbox.bottom(),
            ),
            relative_bbox,
            is_in_air: false,
            velocity: Vec2::new(0., 0.),
            is_facing_left: false,
            x_impulse: 0.,
            run_manager: RunManager::new(),
        }
    }

    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    pub fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    pub fn relative_bbox(&self) -> Rect {
        self.relative_bbox
    }

    pub fn bbox(&self) -> Rect {
        self.relative_bbox.offset(self.pos)
    }

    pub fn process_input_and_physics(
        &mut self,
        config: &Config,
        level: &Level,
        time_since_last_frame: f64,
    ) {
        self.run_manager.update(
            &config,
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

        let prev_bbox = self.bbox();
        self.pos.x += (self.velocity.x + self.x_impulse) * time_since_last_frame as f32;
        self.pos.y += self.velocity.y * time_since_last_frame as f32;

        let mut is_on_any_surface_this_frame = false;

        loop {
            let player_actor = Actor {
                prev_bbox,
                bbox: self.bbox(),
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
                        self.pos += collision.displacement;
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
            self.is_facing_left = self.x_impulse < 0.;
        }
    }

    pub fn sprite<'a>(&self, sprites: &'a GameSprites) -> &'a Sprite {
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

    pub fn draw_debug_rects(&self, sprites: &GameSprites) {
        let sprite = self.sprite(&sprites);

        sprite.draw_debug_rect(self.pos.x, self.pos.y, GREEN);
        draw_rect_lines(&self.bbox(), 2., PURPLE);
    }

    pub fn draw(&self, sprites: &GameSprites, absolute_frame_number: u32) {
        let sprite = self.sprite(&sprites);

        sprite.draw_ex(
            self.pos.x,
            self.pos.y,
            absolute_frame_number % sprite.num_frames(),
            SpriteDrawParams {
                flip_x: self.is_facing_left,
                ..Default::default()
            },
        );
    }
}
