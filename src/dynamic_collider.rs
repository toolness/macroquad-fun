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
    computed_collider: Option<Collider>,
}

impl DynamicColliderComponent {
    pub fn new(relative_collider: RelativeCollider) -> Self {
        DynamicColliderComponent {
            relative_collider,
            ..Default::default()
        }
    }
}

pub struct DynamicColliderSystem {
    /// Cached value of all the dynamic colliders that currently exist.
    /// This is done partly, for efficiency, but also because it's hard
    /// for our physics system to do nested loops over all entities,
    /// given Rust's borrow checker.
    colliders: HashMap<u64, Collider>,
}

impl DynamicColliderSystem {
    pub fn with_capacity(capacity: usize) -> Self {
        DynamicColliderSystem {
            colliders: HashMap::with_capacity(capacity),
        }
    }

    /// Update the DynamicCollider components based on their sprites' current
    /// positions.
    pub fn run(&mut self, entities: &mut EntityMap) {
        update_dynamic_colliders(entities);
        self.colliders.clear();
        for (&id, entity) in entities.iter() {
            if let Some(collider) = get_computed_collider(entity) {
                self.colliders.insert(id, collider);
            }
        }
    }

    pub fn update_dynamic_collider(&mut self, entity_id: u64, entity: &mut Entity) {
        update_dynamic_collider(entity_id, entity, false);
        if let Some(collider) = self.colliders.get_mut(&entity_id) {
            if let Some(computed_collider) = get_computed_collider(entity) {
                *collider = computed_collider;
            }
        }
    }

    pub fn colliders(&self) -> impl Iterator<Item = &Collider> {
        self.colliders.values()
    }
}

fn update_dynamic_collider(id: u64, entity: &mut Entity, update_prev_rect: bool) {
    if let Some(dynamic_collider) = entity.dynamic_collider.as_mut() {
        let rect = entity
            .sprite
            .calculate_absolute_bounding_box(&dynamic_collider.relative_collider.rect);
        let prev_rect = {
            if let Some(computed_collider) = dynamic_collider.computed_collider {
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
        dynamic_collider.computed_collider = Some(Collider {
            rect,
            prev_rect,
            entity_id: Some(id),
            flags: relative.collision_flags,
            velocity: entity.physics.velocity,
            enable_top: relative.enable_top,
            enable_bottom: relative.enable_bottom,
            enable_right: relative.enable_right,
            enable_left: relative.enable_left,
        });
    }
}

fn update_dynamic_colliders(entities: &mut EntityMap) {
    for (&id, entity) in entities.iter_mut() {
        update_dynamic_collider(id, entity, true);
    }
}

fn get_computed_collider(entity: &Entity) -> Option<Collider> {
    entity
        .dynamic_collider
        .as_ref()
        .map(|dc| dc.computed_collider)
        .flatten()
}

pub fn draw_dynamic_collider_debug_rects(entities: &EntityMap) {
    for dynamic_collider in entities.values().filter_map(get_computed_collider) {
        dynamic_collider.draw_debug_rect(PURPLE);
    }
}
