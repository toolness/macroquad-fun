use crate::drawing::draw_rect_lines;
use crate::entity::Entity;
use crate::flying_eye::{create_flying_eye, flying_eye_movement_system};
use crate::mushroom::{create_mushrom, mushroom_movement_system};
use crate::text::draw_level_text;
use crate::time::GameTime;
use crate::{camera::Camera, level::EntityKind};
use macroquad::prelude::*;
use std::collections::HashMap;

use crate::{level::Level, player::Player};

#[derive(PartialEq)]
pub enum FrameResult {
    Ok,
    PlayerDied,
}

pub struct LevelRuntime {
    level: &'static Level,
    entities: HashMap<u64, Entity>,
    player: Player,
    debug_mode: bool,
    camera: Camera,
    next_id: u64,
    time: GameTime,
}

impl LevelRuntime {
    pub fn new(player: Player, level: &'static Level) -> Self {
        let mut instance = LevelRuntime {
            player,
            level,
            entities: HashMap::new(),
            next_id: 1,
            debug_mode: false,
            camera: Camera::new(),
            time: GameTime::new(),
        };
        instance.change_level(&level);
        instance
    }

    fn add_entity(&mut self, entity: Entity) {
        let id = self.new_id();
        self.entities.insert(id, entity);
    }

    pub fn change_level(&mut self, level: &'static Level) {
        self.level = level;
        self.entities.clear();
        self.camera.cut();
        self.spawn_entities();
    }

    fn spawn_entities(&mut self) {
        for entity in self.level.entities.iter() {
            match entity.kind {
                EntityKind::FlyingEye(velocity) => {
                    self.add_entity(create_flying_eye(entity.rect, velocity));
                }
                EntityKind::Mushroom => {
                    self.add_entity(create_mushrom(entity.rect));
                }
                _ => {}
            }
        }
    }

    fn new_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn advance_one_frame(&mut self) -> FrameResult {
        self.time.update();

        if let Some(new_level) = self.player.maybe_switch_levels(&self.level) {
            self.change_level(new_level);
        } else if self.player.fell_off_level(&self.level) {
            return FrameResult::PlayerDied;
        }

        self.camera.update(&self.player, &self.level);

        // Draw environment.
        self.level.draw(&self.camera.rect());

        mushroom_movement_system(&mut self.entities, &self.player, &self.level, &self.time);
        flying_eye_movement_system(&mut self.entities, &self.level, &self.time);

        self.player
            .process_input_and_update(&self.level, &self.entities, &self.time);

        // Draw entities.

        for entity in self.entities.values() {
            entity.sprite.draw_current_frame();
        }

        self.player.sprite_component().draw_current_frame();

        draw_level_text(&self.player, &self.level, &self.camera.rect());

        // Process miscellaneous system input.

        if is_key_pressed(KeyCode::GraveAccent) {
            self.debug_mode = !self.debug_mode;
        }

        if self.debug_mode {
            self.draw_debug_layer();
        }

        return FrameResult::Ok;
    }

    fn draw_debug_layer(&self) {
        let level = self.level;
        self.player.sprite_component().draw_debug_rects();
        for collider in level.iter_colliders(&level.pixel_bounds()) {
            collider.draw_debug_rect(PURPLE);
        }
        draw_rect_lines(
            &level.get_bounding_cell_rect(&self.player.sprite_component().bbox()),
            1.,
            WHITE,
        );
        for entity in self.entities.values() {
            entity.sprite.draw_debug_rects();
        }
        let text = format!("fps: {}", get_fps());
        draw_text(
            &text,
            self.camera.rect().x + 32.,
            self.camera.rect().y + 32.,
            32.0,
            WHITE,
        );
    }
}
