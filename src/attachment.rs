use macroquad::prelude::Vec2;

use crate::{
    collision::{collision_resolution_loop, process_collision},
    entity::{Entity, EntityMap},
    flying_eye::carry_entity,
    level::Level,
    sprite_component::SpriteComponent,
};

#[derive(Default)]
pub struct AttachmentComponent {
    attached_to_entity_id: Option<u64>,
    detached_from_entity_id: Option<u64>,
    pub should_attach: bool,
}

pub fn attachment_system(entities: &mut EntityMap, level: &Level) {
    let entities_to_process: Vec<u64> = entities
        .iter()
        .filter_map(|(&id, entity)| {
            if let Some(attachment) = entity.attachment.as_ref() {
                if attachment.is_attached() || attachment.should_attach {
                    return Some(id);
                }
            }
            return None;
        })
        .collect();

    for id in entities_to_process {
        let mut entity = entities.remove(&id).unwrap();
        let sprite = &mut entity.sprite;
        let attachment = entity.attachment.as_mut().unwrap();
        if let Some(carrier_entity) = attachment.attached_entity(entities) {
            attachment.update_while_attached(&carrier_entity.sprite, level, sprite);
        } else if attachment.should_attach {
            attachment.maybe_attach_to_entity(entities, sprite, &mut entity.velocity);
        }
        entities.insert(id, entity);
    }
}

pub struct AttachableComponent();

impl AttachmentComponent {
    fn maybe_attach_to_entity(
        &mut self,
        entities: &EntityMap,
        passenger: &SpriteComponent,
        velocity: &mut Vec2,
    ) {
        let passenger_bbox = &passenger.bbox();

        for (&id, carrier) in entities.iter() {
            if carrier.attachable.is_none() {
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

    pub fn is_attached(&self) -> bool {
        self.attached_to_entity_id.is_some()
    }

    pub fn detach(&mut self) {
        self.detached_from_entity_id = self.attached_to_entity_id.take();
        assert!(self.detached_from_entity_id.is_some());
    }

    fn attached_entity<'a>(&self, entities: &'a EntityMap) -> Option<&'a Entity> {
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

    fn update_while_attached(
        &mut self,
        carrier: &SpriteComponent,
        level: &Level,
        passenger: &mut SpriteComponent,
    ) {
        let prev_bbox = passenger.bbox();
        carry_entity(&carrier, passenger);

        let mut should_detach = false;

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
            self.detach();
        }
    }
}
