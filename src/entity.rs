use std::{collections::HashMap, fmt::Display};

use uuid::Uuid;

use crate::{
    attachment::{AttachableComponent, AttachmentComponent},
    dynamic_collider::DynamicColliderComponent,
    floor_switch::FloorSwitchComponent,
    flying_eye::FlyingEyeComponent,
    hierarchy::ChildComponent,
    life_transfer::LifeTransferComponent,
    mushroom::MushroomComponent,
    physics::PhysicsComponent,
    pickups::PickupComponent,
    player::PlayerComponent,
    push::PushComponent,
    route::RouteComponent,
    running::RunComponent,
    sprite_component::SpriteComponent,
    steering::SteeringComponent,
    switch::SwitchComponent,
    text::TextComponent,
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
    pub pickup: Option<PickupComponent>,
    pub steering: Option<SteeringComponent>,
    pub life_transfer: Option<LifeTransferComponent>,
    pub text: Option<TextComponent>,
    pub child: Option<ChildComponent>,
    pub iid: Option<Uuid>,
    pub name_for_debugging: Option<&'static str>,
}

impl Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(iid) = self.iid {
            iid.fmt(f)
        } else if let Some(name) = self.name_for_debugging {
            name.fmt(f)
        } else {
            f.write_str("UNKNOWN")
        }
    }
}

pub const MAIN_PLAYER_ENTITY_ID: u64 = 0;

#[derive(Clone)]
pub struct EntityMap {
    map: HashMap<u64, Entity>,
    iid_map: HashMap<Uuid, u64>,
    next_id: u64,
}

impl EntityMap {
    pub fn new_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

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
        if let Some(iid) = entity.iid {
            let result = self.iid_map.insert(iid, id);
            if result.is_some() {
                println!("WARNING: Entity with iid {} already exists!", iid);
            }
        }
    }

    pub fn remove(&mut self, id: u64) -> Option<Entity> {
        let result = self.map.remove(&id);
        if let Some(entity) = result {
            if let Some(iid) = entity.iid {
                self.iid_map.remove(&iid);
            }
        }
        result
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

    pub fn get_id_for_iid(&self, iid: Uuid) -> Option<u64> {
        self.iid_map.get(&iid).copied()
    }

    pub fn get(&self, id: u64) -> Option<&Entity> {
        self.map.get(&id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Entity> {
        self.map.get_mut(&id)
    }

    pub fn main_player(&self) -> &Entity {
        &self.map[&MAIN_PLAYER_ENTITY_ID]
    }

    pub fn main_player_mut(&mut self) -> &mut Entity {
        self.map.get_mut(&MAIN_PLAYER_ENTITY_ID).unwrap()
    }

    pub fn clear_all_except_main_player(&mut self) {
        self.map.retain(|&key, entity| {
            if key == MAIN_PLAYER_ENTITY_ID {
                return true;
            }

            // Note that this only preserves direct children
            // of the player, not all descendants!
            if let Some(child) = entity.child {
                return child.parent == MAIN_PLAYER_ENTITY_ID;
            }

            if let Some(iid) = entity.iid {
                self.iid_map.remove(&iid);
            }

            return false;
        });
    }

    pub fn new_ex(main_player: Entity, capacity: usize) -> Self {
        let mut map = EntityMap {
            map: HashMap::with_capacity(capacity),
            iid_map: HashMap::with_capacity(capacity),
            next_id: 1,
        };
        map.insert(MAIN_PLAYER_ENTITY_ID, main_player);
        map
    }

    /// Temporarily remove the given Entity, call the given function, and then
    /// add the entity back.
    ///
    /// This is useful for situations where we need to be able to mutate an Entity,
    /// but also look at other Entities while mutating it.
    fn with_entity_removed<F: FnOnce(&mut Entity, &mut EntityMap, u64)>(&mut self, id: u64, f: F) {
        let mut entity = self.remove(id).unwrap();
        f(&mut entity, self, id);
        self.insert(id, entity);
    }
}

pub const ENTITY_MAX: usize = 1000;

pub type HeaplessEntityVec = heapless::Vec<u64, ENTITY_MAX>;

pub fn filter_and_process_entities<
    Filter: Fn(&Entity) -> bool,
    Processor: FnMut(&mut Entity, &mut EntityMap, u64),
>(
    entities: &mut EntityMap,
    filter: Filter,
    mut processor: Processor,
) {
    let mut entities_to_process: HeaplessEntityVec = heapless::Vec::new();

    entities_to_process.extend(entities.iter().filter_map(|(id, entity)| {
        if filter(entity) {
            Some(id)
        } else {
            None
        }
    }));

    for &id in entities_to_process.iter() {
        entities.with_entity_removed(id, &mut processor);
    }
}
