use macroquad::prelude::{Rect, PURPLE};

use crate::{collision::Collider, entity::EntityMap};

#[derive(Default)]
pub struct RelativeCollider {
    pub rect: Rect,
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
    colliders: Vec<Collider>,
}

impl DynamicColliderSystem {
    pub fn with_capacity(capacity: usize) -> Self {
        DynamicColliderSystem {
            colliders: Vec::with_capacity(capacity),
        }
    }

    /// Update the DynamicCollider components based on their sprites' current
    /// positions.
    pub fn run(&mut self, entities: &mut EntityMap) {
        update_dynamic_colliders(entities);
        self.colliders.clear();
        self.colliders.extend(get_dynamic_colliders(entities));
    }

    pub fn colliders(&self) -> &Vec<Collider> {
        return &self.colliders;
    }
}

fn update_dynamic_colliders(entities: &mut EntityMap) {
    for entity in entities.values_mut() {
        if let Some(dynamic_collider) = entity.dynamic_collider.as_mut() {
            let rect = entity
                .sprite
                .calculate_absolute_bounding_box(&dynamic_collider.relative_collider.rect);
            let prev_rect = {
                if let Some(computed_collider) = dynamic_collider.computed_collider {
                    computed_collider.rect
                } else {
                    rect
                }
            };
            let relative = &dynamic_collider.relative_collider;
            dynamic_collider.computed_collider = Some(Collider {
                rect,
                prev_rect,
                velocity: entity.physics.velocity,
                enable_top: relative.enable_top,
                enable_bottom: relative.enable_bottom,
                enable_right: relative.enable_right,
                enable_left: relative.enable_left,
            });
        }
    }
}

fn get_dynamic_colliders<'a>(entities: &'a EntityMap) -> impl Iterator<Item = Collider> + 'a {
    entities.values().filter_map(|entity| {
        entity
            .dynamic_collider
            .as_ref()
            .map(|dc| dc.computed_collider)
            .flatten()
    })
}

pub fn draw_dynamic_collider_debug_rects(entities: &EntityMap) {
    for dynamic_collider in get_dynamic_colliders(entities) {
        dynamic_collider.draw_debug_rect(PURPLE);
    }
}
