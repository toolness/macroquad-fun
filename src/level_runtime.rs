use std::collections::HashMap;
use std::fmt::Write;

use crate::attachment::AttachmentSystem;
use crate::config::config;
use crate::crate_entity::create_crate;
use crate::drawing::draw_rect_lines;
use crate::dynamic_collider::DynamicColliderSystem;
use crate::entity::{Entity, EntityMap, EntityProcessor};
use crate::floor_switch::{create_floor_switch, floor_switch_system};
use crate::flying_eye::{create_flying_eye, flying_eye_movement_system};
use crate::fps::FpsCounter;
use crate::input::InputState;
use crate::moving_platform::create_moving_platform;
use crate::mushroom::{create_mushrom, mushroom_movement_system};
use crate::physics::PhysicsSystem;
use crate::player::{
    did_fall_off_level, player_update_system, process_player_input, should_switch_levels,
    teleport_entity,
};
use crate::push::PushSystem;
use crate::route::{draw_route_debug_targets, RouteSystem};
use crate::switch::SwitchSystem;
use crate::text::draw_level_text;
use crate::time::GameTime;
use crate::z_index::ZIndexedDrawingSystem;
use crate::{camera::Camera, level::EntityKind};
use anyhow::Result;
use macroquad::prelude::{PURPLE, WHITE, YELLOW};
use macroquad::text::draw_text;

use crate::level::Level;

const DEBUG_TEXT_CAPACITY: usize = 3000;
const ENTITY_CAPACITY: usize = 200;

#[derive(PartialEq)]
pub enum FrameResult {
    Ok,
    PlayerDied,
}

pub struct LevelRuntime {
    level: &'static Level,
    entities: EntityMap,
    pub debug_mode: bool,
    camera: Camera,
    next_id: u64,
    physics_system: PhysicsSystem,
    route_system: RouteSystem,
    attachment_system: AttachmentSystem,
    switch_system: SwitchSystem,
    push_system: PushSystem,
    dynamic_collider_system: DynamicColliderSystem,
    debug_text_lines: Option<String>,
    z_indexed_drawing_system: ZIndexedDrawingSystem,
    fps: FpsCounter,
}

impl LevelRuntime {
    pub fn new(player: Entity, level: &'static Level) -> Self {
        let mut instance = LevelRuntime {
            level,
            entities: EntityMap::new_ex(player, ENTITY_CAPACITY),
            next_id: 1,
            debug_mode: false,
            camera: Camera::new(),
            physics_system: PhysicsSystem::with_capacity(ENTITY_CAPACITY),
            route_system: RouteSystem {
                processor: EntityProcessor::with_capacity(ENTITY_CAPACITY),
            },
            attachment_system: AttachmentSystem {
                processor: EntityProcessor::with_capacity(ENTITY_CAPACITY),
            },
            push_system: PushSystem {
                processor: EntityProcessor::with_capacity(ENTITY_CAPACITY),
            },
            switch_system: SwitchSystem {
                processor: EntityProcessor::with_capacity(ENTITY_CAPACITY),
            },
            dynamic_collider_system: DynamicColliderSystem::with_capacity(ENTITY_CAPACITY),
            z_indexed_drawing_system: ZIndexedDrawingSystem::with_capacity(ENTITY_CAPACITY),
            debug_text_lines: None,
            fps: Default::default(),
        };
        instance.change_level(&level);
        instance
    }

    fn change_level(&mut self, level: &'static Level) {
        self.level = level;
        self.entities.clear_all_except_player();
        self.spawn_entities();
    }

    fn spawn_entities(&mut self) {
        // Create a mapping from LDtk Entity IIDs to our runtime entity IDs. We'll do this
        // up-front so we can convert EntityRefs in our Entities into entity IDs at spawn time,
        // rather than having to do it every frame.
        let mut iid_id_map: HashMap<&str, u64> = HashMap::with_capacity(self.level.entities.len());
        for entity in self.level.entities.iter() {
            let result = iid_id_map.insert(&entity.iid, self.new_id());
            assert!(
                result.is_none(),
                "All level entities should have unique IIDs"
            );
        }

        for entity in self.level.entities.iter() {
            let opt_instance = match &entity.kind {
                EntityKind::FlyingEye(velocity) => Some(create_flying_eye(entity.rect, *velocity)),
                EntityKind::Mushroom => Some(create_mushrom(entity.rect)),
                EntityKind::MovingPlatform(args) => Some(create_moving_platform(entity.rect, args)),
                EntityKind::Crate => Some(create_crate(entity.rect)),
                EntityKind::FloorSwitch(trigger_entity_iid) => Some(create_floor_switch(
                    entity.rect,
                    trigger_entity_iid
                        .as_ref()
                        .map(|s| iid_id_map[s.iid.as_str()]),
                )),
                EntityKind::PlayerStart(..) | EntityKind::Text(..) => None,
            };
            if let Some(mut instance) = opt_instance {
                instance.iid = Some(&entity.iid);
                self.entities
                    .insert(iid_id_map[entity.iid.as_str()], instance);
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
        if let Some((new_level, new_pos)) = should_switch_levels(&player.sprite, &self.level) {
            teleport_entity(player, new_pos);
            self.change_level(new_level);
            true
        } else {
            false
        }
    }

    pub fn advance_one_frame(&mut self, time: &GameTime, input: &InputState) -> FrameResult {
        self.fps.update(time.now);

        if !self.maybe_switch_level()
            && did_fall_off_level(&self.entities.player().sprite, &self.level)
        {
            return FrameResult::PlayerDied;
        }

        process_player_input(&mut self.entities, time, input);
        self.attachment_system
            .run(&mut self.entities, &self.level, time);
        self.route_system.run(&mut self.entities);
        self.physics_system
            .update_positions(&mut self.entities, time);
        self.dynamic_collider_system.run(&mut self.entities);
        self.push_system.run(&mut self.entities);
        self.switch_system.run(&mut self.entities);
        self.physics_system.resolve_collisions(
            &mut self.entities,
            &self.level,
            &mut self.dynamic_collider_system,
        );
        floor_switch_system(&mut self.entities);
        flying_eye_movement_system(&mut self.entities, time);
        mushroom_movement_system(&mut self.entities, time);
        player_update_system(&mut self.entities, time);

        // Draw stuff.
        self.camera.update(&self.entities.player(), &self.level);
        self.level.draw(&self.camera.rect());
        self.z_indexed_drawing_system
            .draw_entities(&self.entities, &self.level);

        draw_level_text(
            &self.entities.player().sprite,
            &self.level,
            &self.camera.rect(),
        );

        if self.debug_mode {
            self.generate_debug_text()
                .expect("Generating debug text should work!");
            self.draw_debug_layer();
        }

        return FrameResult::Ok;
    }

    fn generate_debug_text(&mut self) -> Result<()> {
        let text = self
            .debug_text_lines
            .get_or_insert_with(|| String::with_capacity(DEBUG_TEXT_CAPACITY));
        text.clear();

        writeln!(text, "fps: {}", self.fps.value())?;
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

    fn draw_debug_layer(&self) {
        let level = self.level;
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

        if let Some(text) = &self.debug_text_lines {
            let font_size = config().debug_text_size;
            let margin = 32.;
            let x = self.camera.rect().x + margin;
            let mut y = self.camera.rect().y + margin;
            for line in text.split("\n") {
                draw_text(line, x, y, font_size, YELLOW);
                y += font_size;
            }
        }
    }
}
