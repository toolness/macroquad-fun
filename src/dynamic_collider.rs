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

pub fn update_dynamic_colliders(entities: &mut EntityMap) {
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

pub fn get_dynamic_colliders<'a>(entities: &'a EntityMap) -> impl Iterator<Item = Collider> + 'a {
    entities.values().filter_map(|entity| {
        if let Some(dynamic_collider) = &entity.dynamic_collider {
            dynamic_collider.computed_collider
        } else {
            None
        }
    })
}

pub fn draw_dynamic_collider_debug_rects(entities: &EntityMap) {
    for dynamic_collider in get_dynamic_colliders(entities) {
        dynamic_collider.draw_debug_rect(PURPLE);
    }
}
