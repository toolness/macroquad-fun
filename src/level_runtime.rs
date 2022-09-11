use crate::camera::center_camera;
use crate::drawing::draw_rect_lines;
use crate::{config::config, text::draw_level_text};
use macroquad::prelude::*;
use std::collections::HashMap;

use crate::{flying_eye::FlyingEye, level::Level, player::Player};

pub struct LevelRuntime {
    level: &'static Level,
    flying_eyes: HashMap<u64, FlyingEye>,
    player: Player,
    debug_mode: bool,
    camera_rect: Rect,
    next_id: u64,
}

impl LevelRuntime {
    pub fn new(player: Player, level: &'static Level) -> Self {
        let mut instance = LevelRuntime {
            player,
            level,
            flying_eyes: HashMap::new(),
            next_id: 1,
            debug_mode: false,
            camera_rect: Default::default(),
        };
        instance.change_level(&level);
        instance
    }

    pub fn add_flying_eye(&mut self, flying_eye: FlyingEye) {
        self.flying_eyes.insert(flying_eye.id(), flying_eye);
    }

    pub fn change_level(&mut self, level: &'static Level) {
        self.level = level;
        self.flying_eyes.clear();
        level.spawn_entities(self);
    }

    pub fn new_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub async fn run(&mut self) {
        let mut last_frame_time = get_time();
        let config = config();

        loop {
            // Keep track of time.
            let now = get_time();
            let absolute_frame_number = (now * 1000.0 / config.ms_per_animation_frame) as u32;
            let time_since_last_frame = now - last_frame_time;

            last_frame_time = now;

            if let Some(new_level) = self.player.maybe_switch_levels(&self.level) {
                self.change_level(new_level);
            }

            let level = self.level;

            // Position the camera.
            self.camera_rect = center_camera(&self.player, &level);

            // Draw environment.
            clear_background(GRAY);
            level.draw(&self.camera_rect);

            // Update entities.
            for flying_eye in self.flying_eyes.values_mut() {
                flying_eye.update(&level, time_since_last_frame);
            }

            self.player.process_input_and_update(
                &self.level,
                &self.flying_eyes,
                time_since_last_frame,
            );

            // Draw entities.

            for flying_eye in self.flying_eyes.values() {
                flying_eye.entity().draw(absolute_frame_number);
            }

            self.player.entity().draw(absolute_frame_number);

            draw_level_text(&self.player, &level, &self.camera_rect);

            // Process miscellaneous system input.

            if is_key_released(KeyCode::Escape) {
                break;
            } else if is_key_pressed(KeyCode::GraveAccent) {
                self.debug_mode = !self.debug_mode;
            }

            if self.debug_mode {
                self.draw_debug_layer();
            }

            // Wait for the next frame.

            next_frame().await;
        }
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
            self.camera_rect.x + 32.,
            self.camera_rect.y + 32.,
            32.0,
            WHITE,
        );
    }
}
