use crate::entity::{filter_and_process_entities, EntityMap};

#[derive(Copy, Clone)]
pub struct ChildComponent {
    /// The ID of the parent entity.
    pub parent: u64,
}

pub fn child_component_system(entities: &mut EntityMap) {
    filter_and_process_entities(
        entities,
        |entity| entity.child.is_some(),
        |entity, entities, _| {
            let child = entity.child.unwrap();
            if let Some(parent) = entities.get(child.parent) {
                entity.sprite.pos = parent.sprite.pos;
                entity.sprite.is_facing_left = parent.sprite.is_facing_left;
            } else {
                println!(
                    "Warning: parent entity with id {} does not exist",
                    child.parent
                );
            }
        },
    );
}
