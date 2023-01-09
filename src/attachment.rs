use macroquad::prelude::{Rect, Vec2};

use crate::{
    audio::play_sound_effect,
    config::config,
    entity::{filter_and_process_entities, Entity, EntityMap},
    game_assets::game_assets,
    level::Level,
    physics::PhysicsComponent,
    sprite_component::SpriteComponent,
    time::GameTime,
};

const CARRY_Y_OFFSET: f32 = 10.0;

pub fn attachment_system(entities: &mut EntityMap, level: &Level, time: &GameTime) {
    filter_and_process_entities(
        entities,
        |entity| {
            if let Some(attachment) = entity.attachment.as_ref() {
                attachment.is_attached() || attachment.should_attach
            } else {
                false
            }
        },
        |entity, entities, _| {
            let sprite = &mut entity.sprite;
            let attachment = entity.attachment.as_mut().unwrap();
            if let Some(carrier_entity) = attachment.attached_entity(entities) {
                attachment.update_while_attached(
                    &carrier_entity.sprite,
                    &carrier_entity.physics,
                    sprite,
                    &mut entity.physics,
                    time,
                );
            } else if attachment.should_attach {
                attachment.maybe_attach_to_entity(entities, sprite, &mut entity.physics, level);
            }
        },
    );
}

#[derive(Default, Copy, Clone)]
pub struct AttachmentComponent {
    attached_to_entity_id: Option<u64>,
    detached_from_entity_id: Option<u64>,
    num_frames_displaced: u32,
    pub should_attach: bool,
}

#[derive(Copy, Clone)]
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

        for (id, carrier) in entities.iter() {
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
                    play_sound_effect(game_assets().attach_sound);
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
        self.attached_to_entity_id
            .map(|id| entities.get(id))
            .flatten()
    }

    pub fn reset(&mut self, physics: &mut PhysicsComponent) {
        if self.is_attached() {
            self.detach(physics);
        }
        self.detached_from_entity_id = None;
    }

    fn update_while_attached(
        &mut self,
        carrier_sprite: &SpriteComponent,
        carrier_physics: &PhysicsComponent,
        passenger_sprite: &mut SpriteComponent,
        passenger_physics: &mut PhysicsComponent,
        time: &GameTime,
    ) {
        if passenger_physics.latest_frame.was_displaced {
            // It's possible that the carrier has also just hit something and
            // is about to change course, so let's give the passenger a
            // bit more time before detaching them.
            self.num_frames_displaced += 1;
            if self.num_frames_displaced > 2 {
                self.detach(passenger_physics);
                return;
            }
        } else {
            self.num_frames_displaced = 0;
        }

        let delta = get_passenger_displacement(&carrier_sprite.bbox(), &passenger_sprite.bbox());
        let max_delta = carrier_physics.velocity.length()
            * time.time_since_last_frame as f32
            * config().attach_velocity_coefficient;

        passenger_sprite.pos += delta.clamp_length_max(max_delta);
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
