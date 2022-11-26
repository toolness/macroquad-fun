use macroquad::prelude::Vec2;

use crate::entity::{filter_and_process_entities, EntityMap};

#[derive(Copy, Clone)]
pub struct ChildComponent {
    /// The ID of the parent entity.
    parent: u64,
    /// The position of the child entity relative to its parent.
    pos: Vec2,
}

pub fn child_component_system(entities: &mut EntityMap) {
    filter_and_process_entities(
        entities,
        |entity| entity.child.is_some(),
        |entity, entities| {
            let child = entity.child.unwrap();
            if let Some(parent) = entities.get(child.parent) {
                entity.sprite.pos = parent.sprite.pos + child.pos;
            } else {
                println!(
                    "Warning: parent entity with id {} does not exist",
                    child.parent
                );
            }
        },
    );
}
