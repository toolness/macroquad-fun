use std::collections::HashMap;

use macroquad::prelude::Vec2;

use crate::{
    collision::{collision_resolution_loop, process_collision},
    flying_eye::FlyingEye,
    level::Level,
    level_runtime::Npc,
    sprite_entity::SpriteEntity,
};

#[derive(Default)]
pub struct Attachment {
    attached_to_npc_id: Option<u64>,
    detached_from_npc_id: Option<u64>,
}

impl Attachment {
    pub fn maybe_attach_to_npc(
        &mut self,
        npcs: &HashMap<u64, Npc>,
        entity: &SpriteEntity,
        velocity: &mut Vec2,
    ) {
        let bbox = &entity.bbox();
        for (&id, npc) in npcs.iter() {
            if let Npc::FlyingEye(flying_eye) = npc {
                if flying_eye.entity().bbox().overlaps(&bbox)
                    && self.detached_from_npc_id != Some(id)
                {
                    self.attached_to_npc_id = Some(id);
                    velocity.x = 0.;
                    velocity.y = 0.;
                    break;
                }
            }
        }
    }

    fn attached_flying_eye<'a>(&self, npcs: &'a HashMap<u64, Npc>) -> Option<&'a FlyingEye> {
        if let Some(id) = self.attached_to_npc_id {
            if let Some(Npc::FlyingEye(flying_eye)) = npcs.get(&id) {
                return Some(&flying_eye);
            }
        }
        None
    }

    pub fn reset(&mut self) {
        self.attached_to_npc_id = None;
        self.detached_from_npc_id = None;
    }

    pub fn update(
        &mut self,
        npcs: &HashMap<u64, Npc>,
        level: &Level,
        entity: &mut SpriteEntity,
        force_detach: bool,
    ) -> bool {
        if let Some(flying_eye) = self.attached_flying_eye(&npcs) {
            self.update_while_attached(flying_eye, level, entity, force_detach);
            true
        } else {
            false
        }
    }

    fn update_while_attached(
        &mut self,
        flying_eye: &FlyingEye,
        level: &Level,
        entity: &mut SpriteEntity,
        force_detach: bool,
    ) {
        let prev_bbox = entity.bbox();
        flying_eye.carry_entity(entity);

        let mut should_detach = force_detach;

        collision_resolution_loop(|| {
            let bbox = entity.bbox();
            for collider in level.iter_colliders(&bbox) {
                if let Some(collision) = process_collision(&collider, &prev_bbox, &bbox) {
                    if collision.displacement != Vec2::ZERO {
                        entity.pos += collision.displacement;
                        should_detach = true;
                        return true;
                    }
                }
            }
            return false;
        });

        if should_detach {
            self.detached_from_npc_id = self.attached_to_npc_id.take();
            assert!(self.detached_from_npc_id.is_some());
        }
    }
}
