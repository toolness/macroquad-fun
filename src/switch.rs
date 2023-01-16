use crate::{
    audio::{play_sound_effect, play_sound_effect_at_volume},
    entity::{filter_and_process_entities, EntityMap},
    game_assets::game_assets,
    level::{EntityKind, Level},
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
    pub has_been_switched_on: bool,
}

pub fn switch_system(entities: &mut EntityMap, level: &Level) {
    filter_and_process_entities(
        entities,
        |entity| entity.switch.is_some(),
        |switch_entity, entities, _| {
            let switch_bbox = &switch_entity.sprite.bbox();
            let mut switch = switch_entity.switch.as_mut().unwrap();
            let mut overlaps_anything = false;
            for (_id, entity) in entities.iter() {
                if (switch_entity.physics.collision_flags & entity.physics.collision_flags)
                    .is_empty()
                {
                    // The collider and the entity can't collide, skip this.
                    continue;
                }
                if entity.sprite.bbox().overlaps(switch_bbox) {
                    overlaps_anything = true;
                    break;
                }
            }
            let was_switched_on = switch.is_switched_on;
            switch.is_switched_on = overlaps_anything;

            if was_switched_on != switch.is_switched_on {
                let has_been_switched_on_before = switch.has_been_switched_on;

                if switch.is_switched_on {
                    switch.has_been_switched_on = true;
                }

                // Now see if our corresponding entity in the level data wants us
                // to trigger anything.
                if let Some(iid) = &switch_entity.iid {
                    if let Some(entity) = level.entities.get(iid) {
                        if let EntityKind::Trigger(args) = &entity.kind {
                            if let Some((sound, volume)) = args.play_sound_effect {
                                if switch.is_switched_on && !has_been_switched_on_before {
                                    play_sound_effect_at_volume(sound, volume);
                                }
                            }
                        }
                    };
                };

                // Now look at our on-entity data and see if we need to trigger anything.
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
                        if try_to_start_route(triggered_entity, !switch.is_switched_on) {
                            play_sound_effect(game_assets().switch_sound);
                        }
                    }
                }
            }
        },
    );
}
