use macroquad::prelude::{Rect, Vec2};

use crate::{game_sprites::game_sprites, sprite_entity::SpriteEntity};

pub struct FlyingEye {
    entity: SpriteEntity,
}

impl FlyingEye {
    pub fn new(start_rect: Rect) -> Self {
        let relative_bbox = game_sprites().flying_eye.flight_bbox;
        let entity = SpriteEntity {
            pos: Vec2::new(
                start_rect.left() - relative_bbox.x,
                start_rect.top() - relative_bbox.y,
            ),
            relative_bbox,
            sprite: Some(&game_sprites().flying_eye.flight),
            ..Default::default()
        };
        FlyingEye { entity }
    }

    pub fn entity(&self) -> &SpriteEntity {
        &self.entity
    }
}
