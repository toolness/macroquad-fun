use macroquad::prelude::{Rect, Vec2};

use crate::{game_sprites::game_sprites, sprite_entity::SpriteEntity};

pub struct Mushroom {
    id: u64,
    entity: SpriteEntity,
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
        Mushroom { id, entity }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn draw(&self, _absolute_frame_number: u32) {
        // TODO: If the player has touched the mushroom, bring it to life.
        self.entity.draw(3);
    }
}
