use anyhow::{anyhow, Result};
use std::path::Path;

use crate::ldtk;

const EXPECTED_JSON_VERSION: &str = "1.1.3";

pub struct Level {
    pub width: i64,
    pub height: i64,
    pub grid_size: i64,
    pub colliders: Vec<i64>,
    pub player_start: (i64, i64),
}

impl Level {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let level_json = std::fs::read_to_string(path.as_ref())?;
        let level: ldtk::Coordinate = serde_json::from_str(level_json.as_str())?;
        if level.json_version != EXPECTED_JSON_VERSION {
            return Err(anyhow!(
                "Expected json_version {}, got {}",
                EXPECTED_JSON_VERSION,
                level.json_version
            ));
        }
        let mut colliders: Option<Vec<i64>> = None;
        let mut player_start: Option<(i64, i64)> = None;
        let mut width: i64 = 0;
        let mut height: i64 = 0;
        let mut grid_size: i64 = 0;
        let layers = level.levels[0].layer_instances.as_ref().unwrap();
        for layer in layers.iter() {
            if layer.identifier == "IntGrid" {
                width = layer.c_wid;
                height = layer.c_hei;
                grid_size = layer.grid_size;
                colliders = Some(layer.int_grid_csv.clone());
            } else if layer.identifier == "Entities" {
                for entity in layer.entity_instances.iter() {
                    if entity.identifier == "PlayerStart" {
                        player_start = Some((entity.px[0], entity.px[1] + entity.height))
                    }
                }
            } else {
                eprintln!("Unexpected layer found: {}", layer.identifier);
            }
        }
        Ok(Level {
            width,
            height,
            grid_size,
            colliders: colliders.ok_or(anyhow!("Couldn't find colliders"))?,
            player_start: player_start.ok_or(anyhow!("Couldn't find PlayerStart"))?,
        })
    }
}
