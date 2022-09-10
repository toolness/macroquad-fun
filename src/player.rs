use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode, Rect, Vec2};

use crate::{
    collision::{collision_resolution_loop, process_collision, Side},
    config::config,
    flying_eye::FlyingEye,
    game_sprites::game_sprites,
    level::Level,
    level_runtime::LevelRuntime,
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
    attached_to_flying_eye_index: Option<usize>,
    detached_from_flying_eye_index: Option<usize>,
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
            attached_to_flying_eye_index: None,
            detached_from_flying_eye_index: None,
        }
    }

    pub fn entity(&self) -> &SpriteEntity {
        &self.entity
    }

    fn maybe_attach_to_flying_eye(&mut self, level_runtime: &LevelRuntime) {
        let bbox = &self.entity.bbox();
        for (index, flying_eye) in level_runtime.flying_eyes.iter().enumerate() {
            if flying_eye.entity().bbox().overlaps(&bbox) {
                if let Some(prev_index) = self.detached_from_flying_eye_index {
                    if prev_index == index {
                        continue;
                    }
                }
                self.attached_to_flying_eye_index = Some(index);
                self.velocity = Vec2::ZERO;
                break;
            }
        }
    }

    fn attached_flying_eye<'a>(&self, level_runtime: &'a LevelRuntime) -> Option<&'a FlyingEye> {
        if let Some(index) = self.attached_to_flying_eye_index {
            level_runtime.flying_eyes.get(index)
        } else {
            None
        }
    }

    fn update_while_attached(
        &mut self,
        flying_eye: &FlyingEye,
        _level_runtime: &LevelRuntime,
        _time_since_last_frame: f64,
    ) {
        let config = config();
        let flyer_bbox = flying_eye.entity().bbox();
        let bbox = self.entity.bbox();
        let y_diff = flyer_bbox.bottom() - config.sprite_scale * 10.0 - bbox.top();
        let x_diff = flyer_bbox.left() - bbox.left();
        self.entity.pos += Vec2::new(x_diff, y_diff);

        if is_key_pressed(KeyCode::Space) {
            self.detached_from_flying_eye_index = self.attached_to_flying_eye_index.take();
            assert!(self.detached_from_flying_eye_index.is_some());
        }
    }

    pub fn process_input_and_update(
        &mut self,
        level_runtime: &LevelRuntime,
        time_since_last_frame: f64,
    ) {
        if let Some(flying_eye) = self.attached_flying_eye(&level_runtime) {
            self.update_while_attached(&flying_eye, &level_runtime, time_since_last_frame);
            return;
        }
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

        collision_resolution_loop(|| {
            let bbox = self.entity.bbox();
            for collider in level_runtime.level.iter_colliders(&bbox) {
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
                        return true;
                    }
                }
            }
            false
        });

        if is_on_any_surface_this_frame {
            // The player just landed (or remains on the ground).
            self.is_in_air = false;
            self.detached_from_flying_eye_index = None;
        } else if !self.is_in_air {
            // The player just fell off a ledge.
            self.is_in_air = true;
        }

        if !self.is_in_air && self.x_impulse != 0. {
            self.entity.is_facing_left = self.x_impulse < 0.;
        }

        if self.is_in_air {
            self.maybe_attach_to_flying_eye(&level_runtime);
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
                self.attached_to_flying_eye_index = None;
                return Some(new_level);
            }
        }
        None
    }
}
