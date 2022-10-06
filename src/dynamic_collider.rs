use std::collections::HashMap;

use macroquad::prelude::{Rect, PURPLE};

use crate::{
    collision::{Collider, CollisionFlags},
    entity::{Entity, EntityMap},
};

#[derive(Default)]
pub struct RelativeCollider {
    pub rect: Rect,
    pub collision_flags: CollisionFlags,
    pub enable_top: bool,
    pub enable_bottom: bool,
    pub enable_right: bool,
    pub enable_left: bool,
}

#[derive(Default)]
pub struct DynamicColliderComponent {
    relative_collider: RelativeCollider,
}

impl DynamicColliderComponent {
    pub fn new(relative_collider: RelativeCollider) -> Self {
        DynamicColliderComponent { relative_collider }
    }
}

pub struct DynamicColliderSystem {
    /// Cached values of all the dynamic colliders that currently exist.
    /// This is done partly, for efficiency, but also because it's hard
    /// for our physics system to do nested loops over all entities,
    /// given Rust's borrow checker.
    colliders: HashMap<u64, Collider>,

    /// This solely exists as an instance variable so we can amortize
    /// allocations across frames.
    colliders_to_remove: Vec<u64>,
}

impl DynamicColliderSystem {
    pub fn with_capacity(capacity: usize) -> Self {
        DynamicColliderSystem {
            colliders: HashMap::with_capacity(capacity),
            colliders_to_remove: Vec::with_capacity(capacity),
        }
    }

    /// Update the DynamicCollider components based on their sprites' current
    /// positions.
    pub fn run(&mut self, entities: &mut EntityMap) {
        // Note that we can't just rebuild self.colliders from scratch every time,
        // because their new values depend on the old values.
        for (&id, entity) in entities.iter_mut() {
            self.update_dynamic_collider_impl(id, entity, true);
        }

        // Now remove any stale colliders that no longer exist.
        self.colliders_to_remove.clear();
        self.colliders_to_remove.extend(
            self.colliders
                .keys()
                .filter(|&id| !entities.contains_key(id)),
        );
        for id in self.colliders_to_remove.iter() {
            self.colliders.remove(id);
        }
    }

    fn update_dynamic_collider_impl(
        &mut self,
        id: u64,
        entity: &mut Entity,
        update_prev_rect: bool,
    ) {
        if let Some(dynamic_collider) = entity.dynamic_collider.as_mut() {
            let rect = entity
                .sprite
                .calculate_absolute_bounding_box(&dynamic_collider.relative_collider.rect);
            let prev_rect = {
                if let Some(computed_collider) = self.colliders.get(&id) {
                    if update_prev_rect {
                        computed_collider.rect
                    } else {
                        computed_collider.prev_rect
                    }
                } else {
                    rect
                }
            };
            let relative = &dynamic_collider.relative_collider;
            self.colliders.insert(
                id,
                Collider {
                    rect,
                    prev_rect,
                    entity_id: Some(id),
                    flags: relative.collision_flags,
                    velocity: entity.physics.velocity,
                    enable_top: relative.enable_top,
                    enable_bottom: relative.enable_bottom,
                    enable_right: relative.enable_right,
                    enable_left: relative.enable_left,
                },
            );
        }
    }

    pub fn update_dynamic_collider(&mut self, entity_id: u64, entity: &mut Entity) {
        self.update_dynamic_collider_impl(entity_id, entity, false);
    }

    pub fn colliders(&self) -> impl Iterator<Item = &Collider> {
        self.colliders.values()
    }

    pub fn draw_debug_rects(&self) {
        for dynamic_collider in self.colliders.values() {
            dynamic_collider.draw_debug_rect(PURPLE);
        }
    }
}
