use std::collections::HashMap;

use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode, Rect, Vec2};

use crate::{
    attachment::Attachment,
    collision::{collision_resolution_loop, process_collision, Side},
    config::config,
    entity::Entity,
    game_sprites::game_sprites,
    level::Level,
    running::RunManager,
    sprite::Sprite,
    sprite_component::SpriteComponent,
    time::GameTime,
    world::world,
};

pub struct Player {
    sprite: SpriteComponent,
    is_in_air: bool,
    velocity: Vec2,
    x_impulse: f32,
    run_manager: RunManager,
    attachment: Attachment,
}

impl Player {
    pub fn new(start_rect: Rect) -> Self {
        Player {
            sprite: SpriteComponent {
                relative_bbox: game_sprites().huntress.idle_bbox,
                ..Default::default()
            }
            .at_bottom_left(&start_rect),
            is_in_air: false,
            velocity: Vec2::new(0., 0.),
            x_impulse: 0.,
            run_manager: RunManager::new(),
            attachment: Default::default(),
        }
    }

    pub fn sprite_component(&self) -> &SpriteComponent {
        &self.sprite
    }

    pub fn process_input_and_update(
        &mut self,
        level: &Level,
        entities: &HashMap<u64, Entity>,
        time: &GameTime,
    ) {
        if !self.attachment.update(
            entities,
            level,
            &mut self.sprite,
            is_key_pressed(KeyCode::Space),
        ) {
            self.unattached_process_input_and_update(level, entities, time)
        }
        self.sprite.update_looping_frame_number(time);
    }

    fn unattached_process_input_and_update(
        &mut self,
        level: &Level,
        entities: &HashMap<u64, Entity>,
        time: &GameTime,
    ) {
        let time_since_last_frame = time.time_since_last_frame;
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

        let prev_bbox = self.sprite.bbox();
        self.sprite.pos.x += (self.velocity.x + self.x_impulse) * time_since_last_frame as f32;
        self.sprite.pos.y += self.velocity.y * time_since_last_frame as f32;

        let mut is_on_any_surface_this_frame = false;

        collision_resolution_loop(|| {
            let bbox = self.sprite.bbox();
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
                        self.sprite.pos += collision.displacement;
                        return true;
                    }
                }
            }
            false
        });

        if is_on_any_surface_this_frame {
            // The player just landed (or remains on the ground).
            self.is_in_air = false;
            self.attachment.reset();
        } else if !self.is_in_air {
            // The player just fell off a ledge.
            self.is_in_air = true;
        }

        if !self.is_in_air && self.x_impulse != 0. {
            self.sprite.is_facing_left = self.x_impulse < 0.;
        }

        if self.is_in_air {
            self.attachment
                .maybe_attach_to_entity(&entities, &self.sprite, &mut self.velocity);
        }

        self.sprite.sprite = Some(self.sprite());
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

    pub fn fell_off_level(&self, level: &Level) -> bool {
        self.sprite.bbox().top() - level.pixel_bounds().bottom() > config().fall_off_level_threshold
    }

    pub fn maybe_switch_levels(&mut self, level: &Level) -> Option<&'static Level> {
        let world = world();
        if !level.contains_majority_of(&self.sprite.bbox()) {
            let world_pos = level.to_world_coords(&self.sprite.pos);
            if let Some((new_level, new_pos)) =
                world.find_level_containing_majority_of(&world_pos, &self.sprite.relative_bbox)
            {
                self.sprite.pos = new_pos;
                self.attachment.reset();
                return Some(new_level);
            }
        }
        None
    }
}
