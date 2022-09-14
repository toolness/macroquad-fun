use macroquad::prelude::{clamp, Rect, Vec2};

use crate::{
    config::config, game_sprites::game_sprites, player::Player, sprite_entity::SpriteEntity,
};

enum MushroomState {
    Dead,
    Rezzing(f64),
    Alive,
}

pub struct Mushroom {
    id: u64,
    entity: SpriteEntity,
    state: MushroomState,
}

impl Mushroom {
    pub fn new(id: u64, start_rect: Rect) -> Self {
        let relative_bbox = game_sprites().mushroom.idle_bbox;
        let entity = SpriteEntity {
            pos: Vec2::new(
                start_rect.left() - relative_bbox.x,
                start_rect.bottom() - relative_bbox.bottom(),
            ),
            relative_bbox,
            sprite: Some(&game_sprites().mushroom.death),
            flip_bbox_when_facing_left: true,
            ..Default::default()
        };
        Mushroom {
            id,
            entity,
            state: MushroomState::Dead,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    fn death_frames(&self) -> u32 {
        game_sprites().mushroom.death.num_frames()
    }

    fn get_rez_animation_frame(&self, time: f64, time_rezzed: f64) -> (u32, bool) {
        let death_frames = self.death_frames();
        let time_since_rez = time - time_rezzed;
        let frames_since_rez = (time_since_rez * 1000.0 / (config().ms_per_animation_frame)) as u32;
        (
            death_frames - 1 - clamp(frames_since_rez, 0, death_frames - 1),
            frames_since_rez >= death_frames,
        )
    }

    pub fn draw(&self, time: f64, absolute_frame_number: u32) {
        match self.state {
            MushroomState::Dead => {
                self.entity.draw(self.death_frames() - 1);
            }
            MushroomState::Rezzing(time_rezzed) => {
                self.entity
                    .draw(self.get_rez_animation_frame(time, time_rezzed).0);
            }
            MushroomState::Alive => {
                self.entity.draw(absolute_frame_number);
            }
        }
    }

    pub fn update(&mut self, player: &Player, time: f64) {
        match self.state {
            MushroomState::Dead => {
                if player.entity().bbox().overlaps(&self.entity.bbox()) {
                    self.state = MushroomState::Rezzing(time);
                }
            }
            MushroomState::Rezzing(time_rezzed) => {
                if self.get_rez_animation_frame(time, time_rezzed).1 {
                    self.state = MushroomState::Alive;
                    self.entity.sprite = Some(&game_sprites().mushroom.idle);
                }
            }
            _ => {}
        }
    }
}
