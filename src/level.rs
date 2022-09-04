use std::collections::HashMap;

use anyhow::{anyhow, Error, Result};
use macroquad::prelude::*;

use crate::{collision::Collider, ldtk};

/// The LDtk version we're using.
const EXPECTED_JSON_VERSION: &str = "1.1.3";

#[derive(Eq, PartialEq)]
pub enum ColliderType {
    Empty,
    Solid,
}

impl ColliderType {
    pub fn from_vec(numbers: &Vec<i64>) -> Result<Vec<ColliderType>> {
        let mut result = Vec::with_capacity(numbers.len());

        for &number in numbers.iter() {
            result.push(number.try_into()?);
        }

        Ok(result)
    }
}

impl TryFrom<i64> for ColliderType {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ColliderType::Empty),
            1 => Ok(ColliderType::Solid),
            _ => Err(anyhow!("Unknown IntGrid value: {}", value)),
        }
    }
}

pub struct World {
    levels: HashMap<String, Level>,
}

impl World {
    pub async fn load(path: &str, scale: f32) -> Result<Self> {
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

        for ldtk_level in world.levels.iter() {
            let level = Level::from_ldtk(&ldtk_level, scale)?;
            levels.insert(level.identifier.clone(), level);
        }

        Ok(World { levels })
    }

    pub fn player_start_bottom_left_in_pixels(&self) -> Option<(&Level, Vec2)> {
        for level in self.levels.values() {
            if let Some(player_start) = level.player_start {
                return Some((
                    &level,
                    Vec2::new(
                        player_start.0 as f32 * level.scale,
                        player_start.1 as f32 * level.scale,
                    ),
                ));
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
    ) -> Option<(&Level, Vec2)> {
        for level in self.levels.values() {
            let local_pos = level.from_world_coords(&world_pos);
            let local_rect = relative_rect.offset(local_pos);
            if level.contains_majority_of(&local_rect) {
                return Some((level, local_pos));
            }
        }

        None
    }
}

pub struct Level {
    /// Unique name for the level.
    identifier: String,

    /// Width in grid cells.
    width: i64,

    /// Height in grid cells.
    height: i64,

    /// Width/height of each grid cell in pixels.
    grid_size: i64,

    /// Where the level exists in world coordinates.
    world_rect: Rect,

    /// Colliders for each grid cell, in row-major order. Corresponds to
    /// an IntGrid layer in LDtk.
    colliders: Vec<ColliderType>,

    /// The bottom-left of corner of where the player starts, in pixels, if
    /// this level declares a player start.
    player_start: Option<(i64, i64)>,

    /// How much we're scaling each pixel by.
    scale: f32,
}

impl Level {
    pub fn from_ldtk(level: &ldtk::Level, scale: f32) -> Result<Self> {
        let mut colliders: Option<Vec<ColliderType>> = None;
        let mut player_start: Option<(i64, i64)> = None;
        let mut width: i64 = 0;
        let mut height: i64 = 0;
        let mut grid_size: i64 = 0;
        let world_rect = Rect::new(
            level.world_x as f32,
            level.world_y as f32,
            level.px_wid as f32,
            level.px_hei as f32,
        );
        let layers = level.layer_instances.as_ref().unwrap();
        for layer in layers.iter() {
            if layer.identifier == "IntGrid" {
                width = layer.c_wid;
                height = layer.c_hei;
                grid_size = layer.grid_size;
                colliders = Some(ColliderType::from_vec(&layer.int_grid_csv)?);
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
            identifier: level.identifier.clone(),
            world_rect,
            width,
            height,
            grid_size,
            colliders: colliders.ok_or(anyhow!("Couldn't find colliders"))?,
            player_start,
            scale,
        })
    }

    pub fn pixel_bounds(&self) -> Rect {
        Rect::new(0., 0., self.width_in_pixels(), self.height_in_pixels())
    }

    pub fn width_in_pixels(&self) -> f32 {
        (self.width * self.grid_size) as f32 * self.scale
    }

    pub fn height_in_pixels(&self) -> f32 {
        (self.height * self.grid_size) as f32 * self.scale
    }

    pub fn contains_majority_of(&self, rect: &Rect) -> bool {
        let level_rect = Rect::new(0., 0., self.width_in_pixels(), self.height_in_pixels());
        if let Some(overlap) = level_rect.intersect(*rect) {
            let total_area = rect.w * rect.h;
            let area_in_our_level = overlap.w * overlap.h;
            return area_in_our_level / total_area >= 0.5;
        }
        false
    }

    fn world_offset(&self) -> Vec2 {
        Vec2::new(
            self.world_rect.x * self.scale,
            self.world_rect.y * self.scale,
        )
    }

    pub fn from_world_coords(&self, coords: &Vec2) -> Vec2 {
        *coords - self.world_offset()
    }

    pub fn to_world_coords(&self, coords: &Vec2) -> Vec2 {
        *coords + self.world_offset()
    }

    pub fn draw(&self) {
        let mut i = 0;
        let scaled_size = self.grid_size as f32 * self.scale;
        for y in 0..self.height {
            for x in 0..self.width {
                if self.colliders[i] == ColliderType::Solid {
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
        self.colliders[(y * self.width + x) as usize] != ColliderType::Empty
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
