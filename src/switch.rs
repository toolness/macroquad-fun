use crate::{
    entity::{EntityMap, EntityProcessor},
    route::try_to_start_route,
};

#[derive(Default, Clone, Copy)]
pub struct SwitchComponent {
    pub is_switched_on: bool,
    pub trigger_entity: Option<u64>,
}

pub fn switch_system(processor: &mut EntityProcessor, entities: &mut EntityMap) {
    processor.filter_and_process_entities(
        entities,
        |entity| entity.switch.is_some(),
        |switch_entity, entities| {
            let switch_bbox = &switch_entity.sprite.bbox();
            let mut switch = switch_entity.switch.as_mut().unwrap();
            let mut overlaps_anything = false;
            for (_id, entity) in entities.iter() {
                if entity.sprite.bbox().overlaps(switch_bbox) {
                    overlaps_anything = true;
                    break;
                }
            }
            let was_switched_on = switch.is_switched_on;
            switch.is_switched_on = overlaps_anything;

            if was_switched_on != switch.is_switched_on {
                if let Some(id) = switch.trigger_entity {
                    if let Some(triggered_entity) = entities.get_mut(id) {
                        try_to_start_route(triggered_entity, !switch.is_switched_on);
                    }
                }
            }
        },
    );
}
