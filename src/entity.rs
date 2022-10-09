use std::{collections::HashMap, fmt::Display};

use crate::{
    attachment::{AttachableComponent, AttachmentComponent},
    dynamic_collider::DynamicColliderComponent,
    floor_switch::FloorSwitchComponent,
    flying_eye::FlyingEyeComponent,
    mushroom::MushroomComponent,
    physics::PhysicsComponent,
    player::PlayerComponent,
    push::PushComponent,
    route::RouteComponent,
    running::RunComponent,
    sprite_component::SpriteComponent,
    switch::SwitchComponent,
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
    pub push: Option<PushComponent>,
    pub switch: Option<SwitchComponent>,
    pub floor_switch: Option<FloorSwitchComponent>,
    pub iid: Option<&'static str>,
}

impl Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.iid.unwrap_or("UNKNOWN")))
    }
}

pub const PLAYER_ENTITY_ID: u64 = 0;

pub trait EntityMapHelpers {
    fn new_ex(player: Entity, capacity: usize) -> Self;
    fn player(&self) -> &Entity;
    fn player_mut(&mut self) -> &mut Entity;
    fn with_entity_removed<F: FnOnce(&mut Entity, &mut EntityMap)>(&mut self, id: u64, f: F);
    fn find_entity_id_with_iid(&self, iid: &str) -> Option<u64>;
}

pub type EntityMap = HashMap<u64, Entity>;

impl EntityMapHelpers for EntityMap {
    fn player(&self) -> &Entity {
        &self[&PLAYER_ENTITY_ID]
    }

    fn player_mut(&mut self) -> &mut Entity {
        self.get_mut(&PLAYER_ENTITY_ID).unwrap()
    }

    fn new_ex(player: Entity, capacity: usize) -> Self {
        let mut map = EntityMap::with_capacity(capacity);
        map.insert(PLAYER_ENTITY_ID, player);
        map
    }

    /// Temporarily remove the given Entity, call the given function, and then
    /// add the entity back.
    ///
    /// This is useful for situations where we need to be able to mutate an Entity,
    /// but also look at other Entities while mutating it.
    fn with_entity_removed<F: FnOnce(&mut Entity, &mut EntityMap)>(&mut self, id: u64, f: F) {
        let mut entity = self.remove(&id).unwrap();
        f(&mut entity, self);
        self.insert(id, entity);
    }

    fn find_entity_id_with_iid(&self, iid: &str) -> Option<u64> {
        // This is O(n), if we use it a lot and have lots of entities, we should make
        // a lookup table or something instead.
        for (&id, entity) in self.iter() {
            if matches!(entity.iid, Some(entity_iid) if entity_iid == iid) {
                return Some(id);
            }
        }
        None
    }
}

pub struct EntityProcessor {
    /// This solely exists as an instance variable so we can amortize
    /// allocations across frames.
    entities_to_process: Vec<u64>,
}

impl EntityProcessor {
    pub fn with_capacity(capacity: usize) -> Self {
        EntityProcessor {
            entities_to_process: Vec::with_capacity(capacity),
        }
    }

    pub fn filter_and_process_entities<
        Filter: Fn(&Entity) -> bool,
        Processor: FnMut(&mut Entity, &mut EntityMap),
    >(
        &mut self,
        entities: &mut EntityMap,
        filter: Filter,
        mut processor: Processor,
    ) {
        self.entities_to_process.clear();
        self.entities_to_process
            .extend(entities.iter().filter_map(
                |(&id, entity)| {
                    if filter(entity) {
                        Some(id)
                    } else {
                        None
                    }
                },
            ));

        for &id in self.entities_to_process.iter() {
            entities.with_entity_removed(id, &mut processor);
        }
    }
}
