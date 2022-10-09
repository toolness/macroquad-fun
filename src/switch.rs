use crate::{
    entity::{EntityMap, EntityMapHelpers, EntityProcessor},
    route::try_to_start_route,
};

#[derive(Default)]
pub struct SwitchComponent {
    pub is_switched_on: bool,
    pub trigger_entity_iid: Option<&'static str>,
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
                let was_switched_on = switch.is_switched_on;
                switch.is_switched_on = overlaps_anything;

                if was_switched_on != switch.is_switched_on {
                    if let Some(iid) = switch.trigger_entity_iid {
                        if let Some(trigger_entity_id) = entities.find_entity_id_with_iid(iid) {
                            if let Some(triggered_entity) = entities.get_mut(&trigger_entity_id) {
                                try_to_start_route(triggered_entity, !switch.is_switched_on);
                            }
                        }
                    }
                }
            },
        );
    }
}
