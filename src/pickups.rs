use macroquad::prelude::Rect;

use crate::{
    entity::{Entity, EntityMap, EntityProcessor},
    game_assets::game_assets,
    physics::PhysicsComponent,
    sprite_component::{Renderer, SpriteComponent},
    time::GameTime,
};

#[derive(Clone, Copy)]
pub enum Pickup {
    Spear,
}

pub type PickupComponent = Pickup;

pub fn create_spear(start_rect: Rect) -> Entity {
    let assets = &game_assets().spear;
    Entity {
        sprite: SpriteComponent {
            relative_bbox: assets.spear_move_bbox,
            renderer: Renderer::Sprite(&assets.spear_move),
            ..Default::default()
        }
        .at_bottom_left(&start_rect),
        physics: PhysicsComponent {
            defies_gravity: true,
            ..Default::default()
        },
        pickup: Some(Pickup::Spear),
        ..Default::default()
    }
}

fn grab_pickup(player_entity: &mut Entity, pickup: Pickup) {
    let mut player = player_entity.player.as_mut().unwrap();
    match pickup {
        Pickup::Spear => {
            player.has_spear = true;
        }
    }
}

pub fn pickup_system(processor: &mut EntityProcessor, entities: &mut EntityMap, time: &GameTime) {
    processor.filter_and_process_entities(
        entities,
        |entity| entity.player.is_some(),
        |player_entity, entities| {
            for (id, entity) in entities.iter() {
                if let Some(pickup) = entity.pickup {
                    if player_entity.sprite.bbox().overlaps(&entity.sprite.bbox()) {
                        grab_pickup(player_entity, pickup);
                        // TODO: Remove pickup!
                    }
                }
            }
        },
    );

    for (_id, entity) in entities.iter_mut() {
        if entity.pickup.is_some() {
            entity.sprite.update_looping_frame_number(time);
        }
    }
}
