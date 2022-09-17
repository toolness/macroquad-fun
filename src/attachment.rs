use std::collections::HashMap;

use macroquad::prelude::Vec2;

use crate::{
    collision::{collision_resolution_loop, process_collision},
    entity::Entity,
    flying_eye::carry_entity,
    level::Level,
    sprite_component::SpriteComponent,
};

#[derive(Default)]
pub struct Attachment {
    attached_to_entity_id: Option<u64>,
    detached_from_entity_id: Option<u64>,
}

impl Attachment {
    pub fn maybe_attach_to_entity(
        &mut self,
        entities: &HashMap<u64, Entity>,
        passenger: &SpriteComponent,
        velocity: &mut Vec2,
    ) {
        let passenger_bbox = &passenger.bbox();

        for (&id, carrier) in entities.iter() {
            if carrier.flying_eye.is_none() {
                continue;
            }
            if carrier.sprite.bbox().overlaps(&passenger_bbox)
                && self.detached_from_entity_id != Some(id)
            {
                self.attached_to_entity_id = Some(id);
                velocity.x = 0.;
                velocity.y = 0.;
                break;
            }
        }
    }

    fn attached_entity<'a>(&self, entities: &'a HashMap<u64, Entity>) -> Option<&'a Entity> {
        if let Some(id) = self.attached_to_entity_id {
            entities.get(&id)
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        self.attached_to_entity_id = None;
        self.detached_from_entity_id = None;
    }

    pub fn update(
        &mut self,
        entities: &HashMap<u64, Entity>,
        level: &Level,
        passenger: &mut SpriteComponent,
        force_detach: bool,
    ) -> bool {
        if let Some(entity) = self.attached_entity(&entities) {
            self.update_while_attached(entity, level, passenger, force_detach);
            true
        } else {
            false
        }
    }

    fn update_while_attached(
        &mut self,
        carrier: &Entity,
        level: &Level,
        passenger: &mut SpriteComponent,
        force_detach: bool,
    ) {
        let prev_bbox = passenger.bbox();
        carry_entity(&carrier.sprite, passenger);

        let mut should_detach = force_detach;

        collision_resolution_loop(|| {
            let bbox = passenger.bbox();
            for collider in level.iter_colliders(&bbox) {
                if let Some(collision) = process_collision(&collider, &prev_bbox, &bbox) {
                    if collision.displacement != Vec2::ZERO {
                        passenger.pos += collision.displacement;
                        should_detach = true;
                        return true;
                    }
                }
            }
            return false;
        });

        if should_detach {
            self.detached_from_entity_id = self.attached_to_entity_id.take();
            assert!(self.detached_from_entity_id.is_some());
        }
    }
}
