use std::cell::RefCell;

use crate::{entity::EntityMap, level::Level};

#[derive(Default)]
pub struct ZIndexComponent {
    value: i32,
}

impl ZIndexComponent {
    pub fn new(value: i32) -> Self {
        ZIndexComponent { value }
    }
}

pub struct ZIndexedDrawingSystem {
    entity_z_indices: RefCell<Vec<(u64, i32)>>,
}

impl ZIndexedDrawingSystem {
    pub fn with_capacity(capacity: usize) -> Self {
        ZIndexedDrawingSystem {
            entity_z_indices: RefCell::new(Vec::with_capacity(capacity)),
        }
    }

    fn update_entity_z_indices(&self, entities: &EntityMap) {
        // This probably isn't terribly performant. Ideally we'd just leverage the
        // GPU's z-buffer here, blitting each sprite with depth information
        // corresponding to their z-index, offloading all this work to the GPU.
        let mut entity_z_indices = self.entity_z_indices.borrow_mut();
        entity_z_indices.clear();
        entity_z_indices.extend(
            entities
                .iter()
                .map(|(id, entity)| (id, entity.z_index.value)),
        );
        entity_z_indices.sort_by(|a, b| a.1.cmp(&b.1));
    }

    pub fn draw_entities(&self, entities: &EntityMap, level: &Level) {
        self.update_entity_z_indices(entities);
        for (id, _) in self.entity_z_indices.borrow().iter() {
            let entity = &entities.get(*id).unwrap();
            entity.sprite.draw_current_frame(level);
        }
    }
}
