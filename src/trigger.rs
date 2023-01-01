use macroquad::prelude::Rect;

use crate::{
    collision::CollisionFlags,
    entity::Entity,
    physics::PhysicsComponent,
    sprite_component::{Renderer, SpriteComponent},
    switch::{SwitchComponent, TriggerType},
};

pub fn create_trigger(rect: Rect, destroy_on_enter: Option<u64>) -> Entity {
    Entity {
        sprite: SpriteComponent {
            renderer: Renderer::Invisible,
            ..Default::default()
        }
        .with_pos_and_size(&rect),
        physics: PhysicsComponent {
            defies_gravity: true,
            // Currently we're only using this for cases where the player
            // triggers the trigger, but we might end up making this an argument
            // that's passed-in at some point.
            collision_flags: CollisionFlags::PLAYER_ONLY,
            ..Default::default()
        },
        switch: Some(SwitchComponent {
            trigger: destroy_on_enter.map(|id| (TriggerType::Destroy, id)),
            ..Default::default()
        }),
        ..Default::default()
    }
}
