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

#[derive(Default, Clone, Copy)]
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

pub struct EntityMap {
    map: HashMap<u64, Entity>,
}

impl EntityMap {
    pub fn iter(&self) -> impl Iterator<Item = (u64, &Entity)> {
        self.map.iter().map(|(&id, entity)| (id, entity))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (u64, &mut Entity)> {
        self.map.iter_mut().map(|(&id, entity)| (id, entity))
    }

    pub fn contains(&self, id: u64) -> bool {
        self.map.contains_key(&id)
    }

    pub fn insert(&mut self, id: u64, entity: Entity) {
        assert!(!self.map.contains_key(&id), "Entity with id already exists");
        self.map.insert(id, entity);
    }

    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn ids(&self) -> impl Iterator<Item = &u64> {
        self.map.keys()
    }

    pub fn get(&self, id: u64) -> Option<&Entity> {
        self.map.get(&id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Entity> {
        self.map.get_mut(&id)
    }

    pub fn player(&self) -> &Entity {
        &self.map[&PLAYER_ENTITY_ID]
    }

    pub fn player_mut(&mut self) -> &mut Entity {
        self.map.get_mut(&PLAYER_ENTITY_ID).unwrap()
    }

    pub fn clear_all_except_player(&mut self) {
        self.map.retain(|&key, _value| key == PLAYER_ENTITY_ID);
    }

    pub fn new_ex(player: Entity, capacity: usize) -> Self {
        let mut map = EntityMap {
            map: HashMap::with_capacity(capacity),
        };
        map.map.insert(PLAYER_ENTITY_ID, player);
        map
    }

    /// Temporarily remove the given Entity, call the given function, and then
    /// add the entity back.
    ///
    /// This is useful for situations where we need to be able to mutate an Entity,
    /// but also look at other Entities while mutating it.
    pub fn with_entity_removed<F: FnOnce(&mut Entity, &mut EntityMap)>(&mut self, id: u64, f: F) {
        let mut entity = self.map.remove(&id).unwrap();
        f(&mut entity, self);
        self.map.insert(id, entity);
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
                |(id, entity)| {
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
