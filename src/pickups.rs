use macroquad::prelude::Rect;

use crate::{
    audio::play_sound_effect,
    config::config,
    entity::{filter_and_process_entities, Entity, EntityMap, HeaplessEntityVec},
    game_assets::game_assets,
    physics::PhysicsComponent,
    sprite_component::{Rotation, SpriteComponent},
    time::GameTime,
};

#[derive(Clone, Copy)]
pub enum PickupType {
    Spear,
    Gem,
}

#[derive(Clone, Copy)]
pub struct PickupComponent {
    kind: PickupType,
    base_y: f32,
}

fn create_pickup(kind: PickupType, mut entity: Entity) -> Entity {
    entity.pickup = Some(PickupComponent {
        kind,
        base_y: entity.sprite.pos.y,
    });
    entity
}

pub fn create_gem(start_rect: Rect) -> Entity {
    let assets = &game_assets().gem;
    create_pickup(
        PickupType::Gem,
        Entity {
            sprite: SpriteComponent {
                base_relative_bbox: assets.gem.frame_rect(),
                sprite: Some(&assets.gem),
                ..Default::default()
            }
            .at_bottom_left(&start_rect),
            physics: PhysicsComponent {
                defies_gravity: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )
}

pub fn create_spear(start_rect: Rect) -> Entity {
    let assets = &game_assets().spear;
    create_pickup(
        PickupType::Spear,
        Entity {
            sprite: SpriteComponent {
                base_relative_bbox: assets.spear_move_bbox,
                sprite: Some(&assets.spear_move),
                rotation: Rotation::Clockwise270,
                ..Default::default()
            }
            .at_bottom_left(&start_rect),
            physics: PhysicsComponent {
                defies_gravity: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )
}

fn grab_pickup(player_entity: &mut Entity, pickup: PickupType) {
    let mut player = player_entity.player.as_mut().unwrap();
    match pickup {
        PickupType::Spear => {
            player.has_spear = true;
        }
        PickupType::Gem => {
            // TODO: Add a gem to the player's inventory.
            play_sound_effect(game_assets().gem.pickup_sound);
        }
    }
}

pub fn pickup_system(entities: &mut EntityMap, time: &GameTime) {
    filter_and_process_entities(
        entities,
        |entity| entity.player.is_some(),
        |player_entity, entities, _| {
            let mut entities_to_remove: HeaplessEntityVec = heapless::Vec::new();
            for (id, entity) in entities.iter() {
                if let Some(pickup) = entity.pickup {
                    if player_entity.sprite.bbox().overlaps(&entity.sprite.bbox()) {
                        grab_pickup(player_entity, pickup.kind);
                        entities_to_remove.push(id).unwrap();
                    }
                }
            }
            for id in entities_to_remove {
                entities.remove(id);
            }
        },
    );

    let config = config();

    for (_id, entity) in entities.iter_mut() {
        if let Some(pickup) = entity.pickup {
            entity.sprite.pos.y = pickup.base_y
                + (config.pickup_float_frequency * time.now as f32).sin()
                    * config.pickup_float_amplitude;
            entity.sprite.update_looping_frame_number(time);
        }
    }
}
