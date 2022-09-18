use macroquad::prelude::{Rect, Vec2};

use crate::{
    config::config,
    entity::{Entity, EntityMap, EntityMapHelpers},
    level::Level,
    physics::PhysicsComponent,
    sprite_component::SpriteComponent,
};

const CARRY_Y_OFFSET: f32 = 10.0;

#[derive(Default)]
pub struct AttachmentComponent {
    attached_to_entity_id: Option<u64>,
    detached_from_entity_id: Option<u64>,
    num_frames_displaced: u32,
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
        entities.with_entity_removed(id, |entity, entities| {
            let sprite = &mut entity.sprite;
            let attachment = entity.attachment.as_mut().unwrap();
            if let Some(carrier_entity) = attachment.attached_entity(entities) {
                attachment.update_while_attached(
                    &carrier_entity.sprite,
                    &carrier_entity.physics,
                    sprite,
                    &mut entity.physics,
                );
            } else if attachment.should_attach {
                attachment.maybe_attach_to_entity(entities, sprite, &mut entity.physics, level);
            }
        });
    }
}

pub struct AttachableComponent();

impl AttachmentComponent {
    fn maybe_attach_to_entity(
        &mut self,
        entities: &EntityMap,
        passenger_sprite: &SpriteComponent,
        passenger_physics: &mut PhysicsComponent,
        level: &Level,
    ) {
        let passenger_bbox = &passenger_sprite.bbox();

        if passenger_physics.defies_gravity {
            // Right now we only support gravity-obeying passengers.
            return;
        }

        for (&id, carrier) in entities.iter() {
            if carrier.attachable.is_none() {
                continue;
            }
            if carrier.sprite.bbox().overlaps(&passenger_bbox)
                && self.detached_from_entity_id != Some(id)
            {
                // Check to see if the passenger will fit on the carrier
                // without running into level geometry.
                let passenger_bbox = passenger_sprite.bbox();
                let delta = get_passenger_displacement(&carrier.sprite.bbox(), &passenger_bbox);
                let projected_passenger_bbox = passenger_bbox.offset(delta);

                if level.is_area_vacant(&projected_passenger_bbox) {
                    self.attached_to_entity_id = Some(id);
                    self.num_frames_displaced = 0;
                    passenger_physics.velocity.x = 0.;
                    passenger_physics.velocity.y = 0.;
                    break;
                }
            }
        }
    }

    pub fn is_attached(&self) -> bool {
        self.attached_to_entity_id.is_some()
    }

    pub fn detach(&mut self, physics: &mut PhysicsComponent) {
        self.detached_from_entity_id = self.attached_to_entity_id.take();
        assert!(self.detached_from_entity_id.is_some());
        physics.velocity = Vec2::ZERO;
        physics.defies_gravity = false;
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
        carrier_sprite: &SpriteComponent,
        carrier_physics: &PhysicsComponent,
        passenger_sprite: &mut SpriteComponent,
        passenger_physics: &mut PhysicsComponent,
    ) {
        if passenger_physics.latest_frame.was_displaced {
            self.num_frames_displaced += 1;
            if self.num_frames_displaced > 2 {
                self.detach(passenger_physics);
                return;
            }
        } else {
            self.num_frames_displaced = 0;
        }

        let delta = get_passenger_displacement(&carrier_sprite.bbox(), &passenger_sprite.bbox());
        passenger_sprite.pos += delta;
        passenger_sprite.is_facing_left = carrier_sprite.is_facing_left;
        passenger_physics.velocity = carrier_physics.velocity;
        passenger_physics.defies_gravity = true;
    }
}

fn get_passenger_displacement(carrier_bbox: &Rect, passenger_bbox: &Rect) -> Vec2 {
    let config = config();
    let y_diff =
        carrier_bbox.bottom() - config.sprite_scale * CARRY_Y_OFFSET - passenger_bbox.top();
    let x_diff = carrier_bbox.left() - passenger_bbox.left();

    Vec2::new(x_diff, y_diff)
}
