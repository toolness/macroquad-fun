use crate::camera::Camera;
use crate::drawing::draw_rect_lines;
use crate::mushroom::Mushroom;
use crate::text::draw_level_text;
use crate::time::GameTime;
use macroquad::prelude::*;
use std::collections::HashMap;

use crate::{flying_eye::FlyingEye, level::Level, player::Player};

#[derive(PartialEq)]
pub enum FrameResult {
    Ok,
    PlayerDied,
}

pub struct LevelRuntime {
    level: &'static Level,
    flying_eyes: HashMap<u64, FlyingEye>,
    mushrooms: HashMap<u64, Mushroom>,
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
            flying_eyes: HashMap::new(),
            mushrooms: HashMap::new(),
            next_id: 1,
            debug_mode: false,
            camera: Camera::new(),
            time: GameTime::new(),
        };
        instance.change_level(&level);
        instance
    }

    pub fn add_flying_eye(&mut self, flying_eye: FlyingEye) {
        self.flying_eyes.insert(flying_eye.id(), flying_eye);
    }

    pub fn add_mushroom(&mut self, mushroom: Mushroom) {
        self.mushrooms.insert(mushroom.id(), mushroom);
    }

    pub fn change_level(&mut self, level: &'static Level) {
        self.level = level;
        self.flying_eyes.clear();
        self.mushrooms.clear();
        self.camera.cut();
        level.spawn_entities(self);
    }

    pub fn new_id(&mut self) -> u64 {
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

        let level = self.level;

        self.camera.update(&self.player, &level);

        // Draw environment.
        clear_background(DARKGRAY);
        level.draw(&self.camera.rect());

        // Update entities.
        for flying_eye in self.flying_eyes.values_mut() {
            flying_eye.update(&level, &self.time);
        }

        for mushroom in self.mushrooms.values_mut() {
            mushroom.update(&self.player, &self.time);
        }

        self.player
            .process_input_and_update(&self.level, &self.flying_eyes, &self.time);

        // Draw entities.

        for flying_eye in self.flying_eyes.values() {
            flying_eye.entity().draw(&self.time);
        }

        for mushroom in self.mushrooms.values() {
            mushroom.draw(&self.time);
        }

        self.player.entity().draw(&self.time);

        draw_level_text(&self.player, &level, &self.camera.rect());

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
        self.player.entity().draw_debug_rects();
        for collider in level.iter_colliders(&level.pixel_bounds()) {
            collider.draw_debug_rect(PURPLE);
        }
        draw_rect_lines(
            &level.get_bounding_cell_rect(&self.player.entity().bbox()),
            1.,
            WHITE,
        );
        for flying_eye in self.flying_eyes.values() {
            flying_eye.entity().draw_debug_rects();
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
