use macroquad::prelude::PURPLE;

use crate::{collision::Collider, entity::EntityMap, sprite_component::SpriteComponent};

#[derive(Default)]
pub struct DynamicColliderComponent {
    pub relative_collider: Collider,
}

pub fn compute_dynamic_collider(
    sprite: &SpriteComponent,
    dynamic_collider: &DynamicColliderComponent,
) -> Collider {
    Collider {
        rect: sprite.calculate_absolute_bounding_box(&dynamic_collider.relative_collider.rect),
        ..dynamic_collider.relative_collider
    }
}

pub fn compute_dynamic_colliders<'a>(
    entities: &'a EntityMap,
) -> impl Iterator<Item = Collider> + 'a {
    entities.values().filter_map(|entity| {
        if let Some(dynamic_collider) = &entity.dynamic_collider {
            Some(compute_dynamic_collider(&entity.sprite, &dynamic_collider))
        } else {
            None
        }
    })
}

pub fn draw_dynamic_collider_debug_rects(entities: &EntityMap) {
    for entity in entities.values() {
        if let Some(dynamic_collider) = &entity.dynamic_collider {
            let collider = compute_dynamic_collider(&entity.sprite, &dynamic_collider);
            collider.draw_debug_rect(PURPLE)
        }
    }
}
