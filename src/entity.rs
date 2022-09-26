use std::collections::HashMap;

use crate::{
    attachment::{AttachableComponent, AttachmentComponent},
    dynamic_collider::DynamicColliderComponent,
    flying_eye::FlyingEyeComponent,
    mushroom::MushroomComponent,
    physics::PhysicsComponent,
    player::PlayerComponent,
    route::RouteComponent,
    running::RunComponent,
    sprite_component::SpriteComponent,
    z_index::ZIndexComponent,
};

#[derive(Default)]
pub struct Entity {
    pub sprite: SpriteComponent,
    pub physics: PhysicsComponent,
    pub z_index: ZIndexComponent,
    pub mushroom: Option<MushroomComponent>,
    pub flying_eye: Option<FlyingEyeComponent>,
    pub attachable: Option<AttachableComponent>,
    pub player: Option<PlayerComponent>,
    pub run: Option<RunComponent>,
    pub attachment: Option<AttachmentComponent>,
    pub dynamic_collider: Option<DynamicColliderComponent>,
    pub route: Option<RouteComponent>,
}

pub const PLAYER_ENTITY_ID: u64 = 0;

pub trait EntityMapHelpers {
    fn with_player(player: Entity) -> Self;
    fn player(&self) -> &Entity;
    fn player_mut(&mut self) -> &mut Entity;
    fn with_entity_removed<F: FnOnce(&mut Entity, &mut EntityMap)>(&mut self, id: u64, f: F);
}

pub const ENTITY_CAPACITY: usize = 200;

pub type EntityMap = HashMap<u64, Entity>;

impl EntityMapHelpers for EntityMap {
    fn player(&self) -> &Entity {
        &self[&PLAYER_ENTITY_ID]
    }

    fn player_mut(&mut self) -> &mut Entity {
        self.get_mut(&PLAYER_ENTITY_ID).unwrap()
    }

    fn with_player(player: Entity) -> Self {
        let mut map = EntityMap::with_capacity(ENTITY_CAPACITY);
        map.insert(PLAYER_ENTITY_ID, player);
        map
    }

    /**
     * Temporarily remove the given Entity, call the given function, and then
     * add the entity back.
     *
     * This is useful for situations where we need to be able to mutate an Entity,
     * but also look at other Entities while mutating it.
     */
    fn with_entity_removed<F: FnOnce(&mut Entity, &mut EntityMap)>(&mut self, id: u64, f: F) {
        let mut entity = self.remove(&id).unwrap();
        f(&mut entity, self);
        self.insert(id, entity);
    }
}
