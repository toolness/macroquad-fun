use macroquad::prelude::PURPLE;

use crate::{collision::Collider, entity::EntityMap};

#[derive(Default)]
pub struct DynamicColliderComponent {
    pub relative_collider: Collider,
    computed_collider: Collider,
}

impl DynamicColliderComponent {
    pub fn new(relative_collider: Collider) -> Self {
        DynamicColliderComponent {
            relative_collider,
            ..Default::default()
        }
    }

    pub fn collider(&self) -> Collider {
        self.computed_collider
    }
}

pub fn update_dynamic_colliders(entities: &mut EntityMap) {
    for entity in entities.values_mut() {
        if let Some(dynamic_collider) = entity.dynamic_collider.as_mut() {
            dynamic_collider.computed_collider = Collider {
                rect: entity
                    .sprite
                    .calculate_absolute_bounding_box(&dynamic_collider.relative_collider.rect),
                ..dynamic_collider.relative_collider
            };
        }
    }
}

pub fn get_dynamic_colliders<'a>(entities: &'a EntityMap) -> impl Iterator<Item = Collider> + 'a {
    entities.values().filter_map(|entity| {
        if let Some(dynamic_collider) = &entity.dynamic_collider {
            Some(dynamic_collider.collider())
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
