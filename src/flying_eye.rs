use macroquad::prelude::{Rect, Vec2};

use crate::{
    collision::{process_collision, Actor},
    config::config,
    game_sprites::game_sprites,
    level::Level,
    sprite_entity::SpriteEntity,
};

pub struct FlyingEye {
    entity: SpriteEntity,
    velocity: Vec2,
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
        FlyingEye {
            entity,
            velocity: Vec2::new(config().flying_eye_speed, 0.),
        }
    }

    pub fn update(&mut self, level: &Level, time_since_last_frame: f64) {
        let prev_bbox = self.entity.bbox();
        self.entity.pos.x += self.velocity.x * time_since_last_frame as f32;
        self.entity.pos.y += self.velocity.y * time_since_last_frame as f32;
        let actor = Actor {
            prev_bbox,
            velocity: self.velocity,
            bbox: self.entity.bbox(),
        };

        for collider in level
            .iter_colliders(&actor.bbox)
            .chain(level.iter_bounds_as_colliders())
        {
            if let Some(collision) = process_collision(&collider, &actor) {
                self.entity.pos += collision.displacement;
                if collision.displacement.x > 0. && self.velocity.x < 0.
                    || collision.displacement.x < 0. && self.velocity.x > 0.
                {
                    self.velocity.x = -self.velocity.x;
                }
                if collision.displacement.y > 0. && self.velocity.y < 0.
                    || collision.displacement.y < 0. && self.velocity.y > 0.
                {
                    self.velocity.y = -self.velocity.y;
                }
            }
        }
        self.entity.is_facing_left = self.velocity.x < 0.;
    }

    pub fn entity(&self) -> &SpriteEntity {
        &self.entity
    }
}
