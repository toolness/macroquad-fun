use crate::{
    entity::{EntityMap, ENTITY_CAPACITY},
    level::Level,
};

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
    entity_z_indices: Vec<(u64, i32)>,
}

impl ZIndexedDrawingSystem {
    pub fn new() -> Self {
        ZIndexedDrawingSystem {
            entity_z_indices: Vec::with_capacity(ENTITY_CAPACITY),
        }
    }

    pub fn draw_entities(&mut self, entities: &EntityMap, level: &Level) {
        // This probably isn't terribly performant. Ideally we'd just leverage the
        // GPU's z-buffer here, blitting each sprite with depth information
        // corresponding to their z-index, offloading all this work to the GPU.
        self.entity_z_indices.clear();
        self.entity_z_indices.extend(
            entities
                .iter()
                .map(|(id, entity)| (*id, entity.z_index.value)),
        );
        self.entity_z_indices.sort_by(|a, b| a.1.cmp(&b.1));

        for (id, _) in self.entity_z_indices.iter() {
            let entity = &entities[&id];
            entity.sprite.draw_current_frame(level);
        }
    }
}
