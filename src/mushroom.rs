use macroquad::prelude::{Rect, Vec2};

use crate::{
    animator::Animator, game_sprites::game_sprites, player::Player, sprite_entity::SpriteEntity,
    time::GameTime,
};

enum MushroomState {
    Dead,
    Rezzing(Animator),
    Alive,
}

pub struct Mushroom {
    id: u64,
    entity: SpriteEntity,
    state: MushroomState,
    dead_frame: u32,
}

impl Mushroom {
    pub fn new(id: u64, start_rect: Rect) -> Self {
        let sprites = &game_sprites().mushroom;
        let relative_bbox = sprites.idle_bbox;
        let death_sprite = &sprites.death;
        let entity = SpriteEntity {
            pos: Vec2::new(
                start_rect.left() - relative_bbox.x,
                start_rect.bottom() - relative_bbox.bottom(),
            ),
            relative_bbox,
            sprite: Some(&death_sprite),
            flip_bbox_when_facing_left: true,
            ..Default::default()
        };
        Mushroom {
            id,
            entity,
            state: MushroomState::Dead,
            dead_frame: death_sprite.last_frame(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn draw(&self, time: &GameTime) {
        match &self.state {
            MushroomState::Dead => {
                self.entity.draw_frame(self.dead_frame);
            }
            MushroomState::Rezzing(animator) => {
                self.entity.draw_frame(animator.get_frame(&time));
            }
            MushroomState::Alive => {
                self.entity.draw(&time);
            }
        }
    }

    pub fn update(&mut self, player: &Player, time: &GameTime) {
        match &self.state {
            MushroomState::Dead => {
                if player.entity().bbox().overlaps(&self.entity.bbox()) {
                    self.state = MushroomState::Rezzing(Animator::new(
                        &game_sprites().mushroom.death,
                        true,
                        &time,
                    ));
                }
            }
            MushroomState::Rezzing(animator) => {
                if animator.is_done(&time) {
                    self.state = MushroomState::Alive;
                    self.entity.sprite = Some(&game_sprites().mushroom.idle);
                }
            }
            _ => {}
        }
    }
}
