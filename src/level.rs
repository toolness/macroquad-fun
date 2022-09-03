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

    fn get_bounding_cell_rect_in_grid(&self, rect: &Rect) -> Rect {
        let grid_scale = self.grid_size as f32 * self.scale;
        let left = (rect.left() / grid_scale).floor();
        let top = (rect.top() / grid_scale).floor();
        let right = (rect.right() / grid_scale).ceil();
        let bottom = (rect.bottom() / grid_scale).ceil();
        Rect::new(left, top, right - left, bottom - top)
    }

    pub fn get_bounding_cell_rect(&self, rect: &Rect) -> Rect {
        let grid_scale = self.grid_size as f32 * self.scale;
        let mut result = self.get_bounding_cell_rect_in_grid(&rect);
        result.x *= grid_scale;
        result.y *= grid_scale;
        result.scale(grid_scale, grid_scale);
        result
    }

    pub fn iter_colliders(&self, bounding_rect: &Rect) -> GridColliderIterator {
        let extents = self.get_bounding_cell_rect_in_grid(&bounding_rect);
        let x_start = extents.left() as i64;
        let x_end = extents.right() as i64;
        let y_start = extents.top() as i64;
        let y_end = extents.bottom() as i64;
        GridColliderIterator {
            level: &self,
            x_start,
            x_end,
            y_end,
            x: x_start,
            y: y_start,
        }
    }
}

pub struct GridColliderIterator<'a> {
    level: &'a Level,
    x_start: i64,
    x_end: i64,
    y_end: i64,
    x: i64,
    y: i64,
}

impl<'a> Iterator for GridColliderIterator<'a> {
    type Item = Collider;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.y > self.y_end {
                return None;
            }
            if self.level.is_occupied_at(self.x, self.y) {
                let x = self.x;
                let y = self.y;
                let scaled_size = self.level.grid_size as f32 * self.level.scale;
                let collider = Collider {
                    enable_top: !self.level.is_occupied_at(x, y - 1),
                    enable_bottom: !self.level.is_occupied_at(x, y + 1),
                    enable_left: !self.level.is_occupied_at(x - 1, y),
                    enable_right: !self.level.is_occupied_at(x + 1, y),
                    rect: Rect::new(
                        (x * self.level.grid_size) as f32 * self.level.scale,
                        (y * self.level.grid_size) as f32 * self.level.scale,
                        scaled_size,
                        scaled_size,
                    ),
                };
                self.advance();
                return Some(collider);
            }
            self.advance();
        }
    }
}

impl<'a> GridColliderIterator<'a> {
    fn advance(&mut self) {
        if self.x < self.x_end {
            self.x += 1;
        } else {
            self.x = self.x_start;
            self.y += 1;
        }
    }
}
