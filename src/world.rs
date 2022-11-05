use anyhow::{anyhow, Result};
use macroquad::prelude::{load_string, Rect, Vec2};
use std::{collections::HashMap, rc::Rc};

use crate::{
    ldtk,
    level::{EntityKind, Level},
};

/// The LDtk version we're using.
const EXPECTED_JSON_VERSION: &str = "1.1.3";

pub struct World {
    levels: HashMap<String, Rc<Level>>,
}

impl World {
    pub async fn load(path: &str) -> Result<Self> {
        let world_json = load_string(&path).await?;
        let world: ldtk::Coordinate = serde_json::from_str(world_json.as_str())?;
        if world.json_version != EXPECTED_JSON_VERSION {
            return Err(anyhow!(
                "Expected json_version {}, got {}",
                EXPECTED_JSON_VERSION,
                world.json_version
            ));
        }
        let mut levels = HashMap::with_capacity(world.levels.len());

        for ldtk_level in world.levels {
            let level = Level::from_ldtk(ldtk_level)?;
            levels.insert(level.identifier.clone(), Rc::new(level));
        }

        Ok(World { levels })
    }

    pub fn player_start(&self, name: &str) -> Option<(Rc<Level>, Rect)> {
        for level in self.levels.values() {
            for entity in level.entities.iter() {
                match &entity.kind {
                    EntityKind::PlayerStart(entity_name) if entity_name == name => {
                        return Some((level.clone(), entity.rect));
                    }
                    _ => {}
                }
            }
        }

        None
    }

    /// Attempt to find the level that contains the majority of the given
    /// rect. If we find one, return a reference to the level and the new
    /// top-left corner of the rect in the level's coordinate system.
    pub fn find_level_containing_majority_of(
        &self,
        world_pos: &Vec2,
        relative_rect: &Rect,
    ) -> Option<(Rc<Level>, Vec2)> {
        for level in self.levels.values() {
            let local_pos = level.from_world_coords(&world_pos);
            let local_rect = relative_rect.offset(local_pos);
            if level.contains_majority_of(&local_rect) {
                return Some((level.clone(), local_pos));
            }
        }

        None
    }
}
