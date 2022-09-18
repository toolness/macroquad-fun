use std::collections::HashMap;

use macroquad::prelude::Vec2;

use crate::{
    attachment::{AttachableComponent, AttachmentComponent},
    flying_eye::FlyingEyeComponent,
    mushroom::MushroomComponent,
    player::PlayerComponent,
    running::RunComponent,
    sprite_component::SpriteComponent,
};

#[derive(Default)]
pub struct Entity {
    pub sprite: SpriteComponent,
    pub velocity: Vec2,
    pub mushroom: Option<MushroomComponent>,
    pub flying_eye: Option<FlyingEyeComponent>,
    pub attachable: Option<AttachableComponent>,
    pub player: Option<PlayerComponent>,
    pub run: Option<RunComponent>,
    pub attachment: Option<AttachmentComponent>,
}

pub const PLAYER_ENTITY_ID: u64 = 0;

pub trait EntityMapHelpers {
    fn with_player(player: Entity) -> Self;
    fn player(&self) -> &Entity;
    fn player_mut(&mut self) -> &mut Entity;
}

pub type EntityMap = HashMap<u64, Entity>;

impl EntityMapHelpers for EntityMap {
    fn player(&self) -> &Entity {
        &self[&PLAYER_ENTITY_ID]
    }

    fn player_mut(&mut self) -> &mut Entity {
        self.get_mut(&PLAYER_ENTITY_ID).unwrap()
    }

    fn with_player(player: Entity) -> Self {
        let mut map = EntityMap::new();
        map.insert(PLAYER_ENTITY_ID, player);
        map
    }
}
