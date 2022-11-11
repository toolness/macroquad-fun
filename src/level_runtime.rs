use std::collections::HashMap;
use std::fmt::Write;
use std::rc::Rc;

use crate::attachment::attachment_system;
use crate::crate_entity::create_crate;
use crate::drawing::draw_rect_lines;
use crate::dynamic_collider::{DynamicColliderSystem, SavedDynamicColliderSystem};
use crate::entity::{Entity, EntityMap, EntityProcessor};
use crate::floor_switch::{create_floor_switch, floor_switch_system};
use crate::flying_eye::{create_flying_eye, flying_eye_movement_system};
use crate::input::InputState;
use crate::moving_platform::create_moving_platform;
use crate::mushroom::{create_mushrom, mushroom_movement_system};
use crate::physics::PhysicsSystem;
use crate::pickups::{create_spear, pickup_system};
use crate::player::{
    did_fall_off_level, player_update_system, process_player_input, should_switch_levels,
    teleport_entity,
};
use crate::push::push_system;
use crate::route::{draw_route_debug_targets, route_system};
use crate::switch::switch_system;
use crate::text::draw_level_text;
use crate::time::GameTime;
use crate::world::World;
use crate::z_index::ZIndexedDrawingSystem;
use crate::{camera::Camera, level::EntityKind};
use anyhow::Result;
use macroquad::prelude::{PURPLE, WHITE};
use uuid::Uuid;

use crate::level::Level;

const ENTITY_CAPACITY: usize = 200;

#[derive(PartialEq)]
pub enum FrameResult {
    Ok,
    PlayerDied,
}

#[derive(Clone)]
pub struct SavedLevelRuntime {
    level: Rc<Level>,
    world: Rc<World>,
    entities: EntityMap,
    camera: Camera,
    next_id: u64,
    dynamic_collider_system: SavedDynamicColliderSystem,
}

pub struct LevelRuntime {
    level: Rc<Level>,
    world: Rc<World>,
    entities: EntityMap,
    camera: Camera,
    next_id: u64,
    physics_system: PhysicsSystem,
    dynamic_collider_system: DynamicColliderSystem,
    z_indexed_drawing_system: ZIndexedDrawingSystem,
    entity_processor: EntityProcessor,
}

impl LevelRuntime {
    pub fn new(player: Entity, level: Rc<Level>, world: Rc<World>) -> Self {
        let mut instance = Self::from_saved(SavedLevelRuntime {
            level: level.clone(),
            world,
            entities: EntityMap::new_ex(player, ENTITY_CAPACITY),
            camera: Camera::new(),
            next_id: 1,
            dynamic_collider_system: SavedDynamicColliderSystem::with_capacity(ENTITY_CAPACITY),
        });
        instance.change_level(level);
        instance
    }

    pub fn from_saved(saved: SavedLevelRuntime) -> Self {
        LevelRuntime {
            level: saved.level,
            world: saved.world,
            entities: saved.entities,
            next_id: saved.next_id,
            camera: saved.camera,
            physics_system: PhysicsSystem::with_capacity(ENTITY_CAPACITY),
            dynamic_collider_system: DynamicColliderSystem::from_saved(
                saved.dynamic_collider_system,
            ),
            z_indexed_drawing_system: ZIndexedDrawingSystem::with_capacity(ENTITY_CAPACITY),
            entity_processor: EntityProcessor::with_capacity(ENTITY_CAPACITY),
        }
    }

    pub fn save(&self) -> SavedLevelRuntime {
        SavedLevelRuntime {
            level: self.level.clone(),
            world: self.world.clone(),
            entities: self.entities.clone(),
            camera: self.camera,
            next_id: self.next_id,
            dynamic_collider_system: self.dynamic_collider_system.save(),
        }
    }

    fn change_level(&mut self, level: Rc<Level>) {
        self.level = level;
        self.entities.clear_all_except_player();
        self.spawn_entities();
    }

    fn spawn_entities(&mut self) {
        // Create a mapping from LDtk Entity IIDs to our runtime entity IDs. We'll do this
        // up-front so we can convert EntityRefs in our Entities into entity IDs at spawn time,
        // rather than having to do it every frame.
        let mut iid_id_map: HashMap<Uuid, u64> = HashMap::with_capacity(self.level.entities.len());
        let level = self.level.clone();
        for entity in level.entities.iter() {
            let result = iid_id_map.insert(entity.iid, self.new_id());
            assert!(
                result.is_none(),
                "All level entities should have unique IIDs"
            );
        }

        for entity in level.entities.iter() {
            let opt_instance = match &entity.kind {
                EntityKind::FlyingEye(velocity) => Some(create_flying_eye(entity.rect, *velocity)),
                EntityKind::Spear => Some(create_spear(entity.rect)),
                EntityKind::Mushroom => Some(create_mushrom(entity.rect)),
                EntityKind::MovingPlatform(args) => Some(create_moving_platform(entity.rect, args)),
                EntityKind::Crate => Some(create_crate(entity.rect)),
                EntityKind::FloorSwitch(trigger_entity_iid) => Some(create_floor_switch(
                    entity.rect,
                    trigger_entity_iid.as_ref().map(|s| iid_id_map[&s.iid]),
                )),
                EntityKind::PlayerStart(..) | EntityKind::Text(..) => None,
            };
            if let Some(mut instance) = opt_instance {
                instance.iid = Some(entity.iid);
                self.entities.insert(iid_id_map[&entity.iid], instance);
            }
        }
    }

    fn new_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn maybe_switch_level(&mut self) -> bool {
        let player = self.entities.player_mut();
        if let Some((new_level, new_pos)) =
            should_switch_levels(&player.sprite, &self.level, &self.world)
        {
            teleport_entity(player, new_pos);
            self.change_level(new_level);
            true
        } else {
            false
        }
    }

    pub fn advance_one_frame(&mut self, time: &GameTime, input: &InputState) -> FrameResult {
        if !self.maybe_switch_level()
            && did_fall_off_level(&self.entities.player().sprite, &self.level)
        {
            return FrameResult::PlayerDied;
        }

        process_player_input(&mut self.entities, time, input);
        attachment_system(
            &mut self.entity_processor,
            &mut self.entities,
            &self.level,
            time,
        );
        route_system(&mut self.entity_processor, &mut self.entities);
        self.physics_system
            .update_positions(&mut self.entities, time);
        self.dynamic_collider_system.run(&mut self.entities);
        push_system(&mut self.entity_processor, &mut self.entities);
        switch_system(&mut self.entity_processor, &mut self.entities);
        self.physics_system.resolve_collisions(
            &mut self.entities,
            &self.level,
            &mut self.dynamic_collider_system,
        );
        floor_switch_system(&mut self.entities);
        flying_eye_movement_system(&mut self.entities, time);
        mushroom_movement_system(&mut self.entities, time);
        pickup_system(&mut self.entity_processor, &mut self.entities, time);
        player_update_system(&mut self.entities, time);

        self.camera.update(&self.entities.player(), &self.level);

        return FrameResult::Ok;
    }

    pub fn draw(&self) {
        self.camera.with_active(|| {
            self.level.draw(&self.camera.rect());
            self.z_indexed_drawing_system
                .draw_entities(&self.entities, &self.level);
        });

        draw_level_text(&self.entities.player().sprite, &self.level);
    }

    pub fn generate_debug_text(&self, text: &mut String) -> Result<()> {
        let entity_size = std::mem::size_of::<Entity>();
        writeln!(
            text,
            "entities: {} ({} bytes each)",
            self.entities.len(),
            entity_size,
        )?;
        writeln!(
            text,
            "entity map capacity: {} ({} bytes total)",
            self.entities.capacity(),
            self.entities.capacity() * entity_size,
        )?;
        Ok(())
    }

    pub fn draw_debug_layer(&self) {
        self.camera.with_active(|| {
            let level = &self.level;
            for collider in level.iter_colliders(&level.pixel_bounds()) {
                collider.draw_debug_rect(PURPLE);
            }
            self.dynamic_collider_system.draw_debug_rects();
            draw_route_debug_targets(&self.entities);
            draw_rect_lines(
                &level.get_bounding_cell_rect(&self.entities.player().sprite.bbox()),
                1.,
                WHITE,
            );
            for (_id, entity) in self.entities.iter() {
                entity.sprite.draw_debug_rects();
            }
            self.camera.draw_debug_info();
        });
    }
}
