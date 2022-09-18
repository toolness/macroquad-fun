use crate::attachment::attachment_system;
use crate::drawing::draw_rect_lines;
use crate::entity::{Entity, EntityMap};
use crate::flying_eye::{create_flying_eye, flying_eye_movement_system};
use crate::mushroom::{create_mushrom, mushroom_movement_system};
use crate::player::{
    did_fall_off_level, process_player_input_and_update, should_switch_levels, teleport_entity,
};
use crate::text::draw_level_text;
use crate::time::GameTime;
use crate::{camera::Camera, level::EntityKind};
use macroquad::prelude::*;

use crate::level::Level;

#[derive(PartialEq)]
pub enum FrameResult {
    Ok,
    PlayerDied,
}

pub struct LevelRuntime {
    level: &'static Level,
    entities: EntityMap,
    debug_mode: bool,
    camera: Camera,
    next_id: u64,
    time: GameTime,
}

const PLAYER_ENTITY_ID: u64 = 0;

impl LevelRuntime {
    pub fn new(player: Entity, level: &'static Level) -> Self {
        let mut instance = LevelRuntime {
            level,
            entities: EntityMap::new(),
            next_id: 1,
            debug_mode: false,
            camera: Camera::new(),
            time: GameTime::new(),
        };
        instance.entities.insert(PLAYER_ENTITY_ID, player);
        instance.change_level(&level);
        instance
    }

    fn add_entity(&mut self, entity: Entity) {
        let id = self.new_id();
        self.entities.insert(id, entity);
    }

    fn change_level(&mut self, level: &'static Level) {
        self.level = level;
        self.entities.retain(|&key, _value| key == PLAYER_ENTITY_ID);
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

        let mut player = self.entities.remove(&PLAYER_ENTITY_ID).unwrap();

        if let Some((new_level, new_pos)) = should_switch_levels(&player.sprite, &self.level) {
            teleport_entity(&mut player, new_pos);
            self.change_level(new_level);
        } else if did_fall_off_level(&player.sprite, &self.level) {
            return FrameResult::PlayerDied;
        }

        self.camera.update(&player.sprite, &self.level);

        // Draw environment.
        self.level.draw(&self.camera.rect());

        mushroom_movement_system(&mut self.entities, &player.sprite, &self.level, &self.time);
        flying_eye_movement_system(&mut self.entities, &self.level, &self.time);

        process_player_input_and_update(&mut player, &self.level, &self.entities, &self.time);

        // Draw entities.

        for entity in self.entities.values() {
            entity.sprite.draw_current_frame();
        }

        player.sprite.draw_current_frame();

        draw_level_text(&player.sprite, &self.level, &self.camera.rect());

        // Process miscellaneous system input.

        if is_key_pressed(KeyCode::GraveAccent) {
            self.debug_mode = !self.debug_mode;
        }

        self.entities.insert(PLAYER_ENTITY_ID, player);

        attachment_system(&mut self.entities, &self.level);

        if self.debug_mode {
            self.draw_debug_layer();
        }

        return FrameResult::Ok;
    }

    fn draw_debug_layer(&self) {
        let level = self.level;
        for collider in level.iter_colliders(&level.pixel_bounds()) {
            collider.draw_debug_rect(PURPLE);
        }
        draw_rect_lines(
            &level.get_bounding_cell_rect(&self.entities[&PLAYER_ENTITY_ID].sprite.bbox()),
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
