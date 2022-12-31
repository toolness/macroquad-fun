use crate::{
    entity::{filter_and_process_entities, EntityMap},
    route::try_to_start_route,
};

#[derive(Clone, Copy)]
pub enum TriggerType {
    ToggleRoute,
    Destroy,
}

#[derive(Default, Clone, Copy)]
pub struct SwitchComponent {
    pub is_switched_on: bool,
    pub trigger: Option<(TriggerType, u64)>,
}

pub fn switch_system(entities: &mut EntityMap) {
    filter_and_process_entities(
        entities,
        |entity| entity.switch.is_some(),
        |switch_entity, entities, _| {
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
                let Some((trigger_type, id)) = switch.trigger else {
                    return
                };
                let Some(triggered_entity) = entities.get_mut(id) else {
                    return
                };
                match trigger_type {
                    TriggerType::Destroy => {
                        entities.remove(id);
                    }
                    TriggerType::ToggleRoute => {
                        try_to_start_route(triggered_entity, !switch.is_switched_on);
                    }
                }
            }
        },
    );
}
