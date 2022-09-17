use crate::drawing::draw_rect_lines;
use crate::mushroom::Mushroom;
use crate::sprite_entity::SpriteEntity;
use crate::text::draw_level_text;
use crate::time::GameTime;
use crate::{camera::Camera, level::EntityKind};
use macroquad::prelude::*;
use std::collections::HashMap;

use crate::{flying_eye::FlyingEye, level::Level, player::Player};

#[derive(PartialEq)]
pub enum FrameResult {
    Ok,
    PlayerDied,
}

pub enum Npc {
    FlyingEye(FlyingEye),
    Mushroom(Mushroom),
}

fn iter_entities<'a>(npcs: &'a HashMap<u64, Npc>) -> impl Iterator<Item = &'a SpriteEntity> {
    npcs.values().map(|npc| match npc {
        Npc::FlyingEye(flying_eye) => flying_eye.entity(),
        Npc::Mushroom(mushroom) => mushroom.entity(),
    })
}

pub struct LevelRuntime {
    level: &'static Level,
    npcs: HashMap<u64, Npc>,
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
            npcs: HashMap::new(),
            next_id: 1,
            debug_mode: false,
            camera: Camera::new(),
            time: GameTime::new(),
        };
        instance.change_level(&level);
        instance
    }

    fn add_npc(&mut self, npc: Npc) {
        let id = self.new_id();
        self.npcs.insert(id, npc);
    }

    pub fn change_level(&mut self, level: &'static Level) {
        self.level = level;
        self.npcs.clear();
        self.camera.cut();
        self.spawn_entities();
    }

    fn spawn_entities(&mut self) {
        for entity in self.level.entities.iter() {
            match entity.kind {
                EntityKind::FlyingEye(velocity) => {
                    self.add_npc(Npc::FlyingEye(FlyingEye::new(entity.rect, velocity)));
                }
                EntityKind::Mushroom => {
                    self.add_npc(Npc::Mushroom(Mushroom::new(entity.rect)));
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

        for npc in self.npcs.values_mut() {
            match npc {
                Npc::FlyingEye(flying_eye) => {
                    flying_eye.update(&self.level, &self.time);
                }
                Npc::Mushroom(mushroom) => {
                    mushroom.update(&self.player, &self.level, &self.time);
                }
            }
        }

        self.player
            .process_input_and_update(&self.level, &self.npcs, &self.time);

        // Draw entities.

        for npc in iter_entities(&self.npcs) {
            npc.draw_current_frame();
        }

        self.player.entity().draw_current_frame();

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
        self.player.entity().draw_debug_rects();
        for collider in level.iter_colliders(&level.pixel_bounds()) {
            collider.draw_debug_rect(PURPLE);
        }
        draw_rect_lines(
            &level.get_bounding_cell_rect(&self.player.entity().bbox()),
            1.,
            WHITE,
        );
        for entity in iter_entities(&self.npcs) {
            entity.draw_debug_rects();
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
