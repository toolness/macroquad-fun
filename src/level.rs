use anyhow::{anyhow, Result};
use macroquad::prelude::*;
use std::path::Path;

use crate::{collision::Collider, ldtk};

const EXPECTED_JSON_VERSION: &str = "1.1.3";

pub struct Level {
    width: i64,
    height: i64,
    grid_size: i64,
    colliders: Vec<i64>,
    player_start: (i64, i64),
    scale: f32,
}

impl Level {
    pub fn load<P: AsRef<Path>>(path: P, scale: f32) -> Result<Self> {
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
            scale,
        })
    }

    pub fn width_in_pixels(&self) -> f32 {
        (self.width * self.grid_size) as f32 * self.scale
    }

    pub fn height_in_pixels(&self) -> f32 {
        (self.height * self.grid_size) as f32 * self.scale
    }

    pub fn player_start_bottom_left_in_pixels(&self) -> Vec2 {
        Vec2::new(
            self.player_start.0 as f32 * self.scale,
            self.player_start.1 as f32 * self.scale,
        )
    }

    pub fn draw(&self) {
        let mut i = 0;
        let scaled_size = self.grid_size as f32 * self.scale;
        for y in 0..self.height {
            for x in 0..self.width {
                if self.colliders[i] == 1 {
                    draw_rectangle(
                        (x * self.grid_size) as f32 * self.scale,
                        (y * self.grid_size) as f32 * self.scale,
                        scaled_size,
                        scaled_size,
                        DARKGRAY,
                    )
                }
                i += 1;
            }
        }
    }

    fn is_occupied_at(&self, x: i64, y: i64) -> bool {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return false;
        }
        self.colliders[(y * self.height + x) as usize] == 1
    }

    pub fn get_bounding_cell_rect(&self, rect: &Rect) -> Rect {
        let grid_scale = self.grid_size as f32 * self.scale;
        let left = (rect.left() / grid_scale).floor() * grid_scale;
        let top = (rect.top() / grid_scale).floor() * grid_scale;
        let right = (rect.right() / grid_scale).ceil() * grid_scale;
        let bottom = (rect.bottom() / grid_scale).ceil() * grid_scale;

        Rect::new(left, top, right - left, bottom - top)
    }

    pub fn get_all_colliders(&self) -> Vec<Collider> {
        let mut result = Vec::new();

        let mut i = 0;
        let scaled_size = self.grid_size as f32 * self.scale;
        for y in 0..self.height {
            for x in 0..self.width {
                if self.colliders[i] == 1 {
                    result.push(Collider {
                        enable_top: !self.is_occupied_at(x, y - 1),
                        enable_bottom: !self.is_occupied_at(x, y + 1),
                        enable_left: !self.is_occupied_at(x - 1, y),
                        enable_right: !self.is_occupied_at(x + 1, y),
                        rect: Rect::new(
                            (x * self.grid_size) as f32 * self.scale,
                            (y * self.grid_size) as f32 * self.scale,
                            scaled_size,
                            scaled_size,
                        ),
                    });
                }
                i += 1;
            }
        }

        result
    }
}
