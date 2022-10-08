use crate::entity::{EntityMap, EntityProcessor};

#[derive(Default)]
pub struct SwitchComponent {
    pub is_switched_on: bool,
}

pub struct SwitchSystem {
    pub processor: EntityProcessor,
}

impl SwitchSystem {
    pub fn run(&mut self, entities: &mut EntityMap) {
        self.processor.filter_and_process_entities(
            entities,
            |entity| entity.switch.is_some(),
            |switch_entity, entities| {
                let switch_bbox = &switch_entity.sprite.bbox();
                let mut switch = switch_entity.switch.as_mut().unwrap();
                let mut overlaps_anything = false;
                for entity in entities.values() {
                    if entity.sprite.bbox().overlaps(switch_bbox) {
                        overlaps_anything = true;
                        break;
                    }
                }
                switch.is_switched_on = overlaps_anything;
            },
        );
    }
}
