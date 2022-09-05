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
            for entity in level.entities.iter() {
                if entity.kind == EntityKind::PlayerStart {
                    return Some((&level, Vec2::new(entity.rect.left(), entity.rect.bottom())));
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
    grid_size: f32,

    /// Where the level exists in world coordinates.
    world_rect: Rect,

    /// Colliders for each grid cell, in row-major order. Corresponds to
    /// an IntGrid layer in LDtk.
    colliders: Vec<ColliderType>,

    /// Various other entities in the level.
    entities: Vec<Entity>,
}

pub struct Entity {
    /// The type of entity.
    kind: EntityKind,

    /// The entity's position in pixel coordinates.
    rect: Rect,
}

#[derive(Eq, PartialEq)]
pub enum EntityKind {
    PlayerStart,
    Text(String),
}

impl Level {
    pub fn from_ldtk(level: &ldtk::Level, scale: f32) -> Result<Self> {
        let mut colliders: Option<Vec<ColliderType>> = None;
        let mut width: i64 = 0;
        let mut height: i64 = 0;
        let mut grid_size: f32 = 0.;
        let world_rect = Rect::new(
            level.world_x as f32 * scale,
            level.world_y as f32 * scale,
            level.px_wid as f32 * scale,
            level.px_hei as f32 * scale,
        );
        let layers = level.layer_instances.as_ref().unwrap();
        let mut entities = vec![];
        for layer in layers.iter() {
            if layer.identifier == "IntGrid" {
                width = layer.c_wid;
                height = layer.c_hei;
                grid_size = layer.grid_size as f32 * scale;
                colliders = Some(ColliderType::from_vec(&layer.int_grid_csv)?);
            } else if layer.identifier == "Entities" {
                for entity in layer.entity_instances.iter() {
                    let rect = Rect::new(
                        entity.px[0] as f32 * scale,
                        entity.px[1] as f32 * scale,
                        entity.width as f32 * scale,
                        entity.height as f32 * scale,
                    );
                    let kind: EntityKind;
                    if entity.identifier == "PlayerStart" {
                        kind = EntityKind::PlayerStart;
                    } else if entity.identifier == "Text" {
                        kind = EntityKind::Text(entity.get_string_field_instance("text")?);
                    } else {
                        eprintln!("Unexpected entity found: {}", entity.identifier);
                        continue;
                    }
                    entities.push(Entity { kind, rect });
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
            entities,
        })
    }

    pub fn pixel_bounds(&self) -> Rect {
        Rect::new(0., 0., self.width_in_pixels(), self.height_in_pixels())
    }

    pub fn width_in_pixels(&self) -> f32 {
        self.width as f32 * self.grid_size
    }

    pub fn height_in_pixels(&self) -> f32 {
        self.height as f32 * self.grid_size
    }

    pub fn contains_majority_of(&self, rect: &Rect) -> bool {
        if let Some(overlap) = self.pixel_bounds().intersect(*rect) {
            let total_area = rect.w * rect.h;
            let area_in_our_level = overlap.w * overlap.h;
            return area_in_our_level / total_area >= 0.5;
        }
        false
    }

    pub fn from_world_coords(&self, coords: &Vec2) -> Vec2 {
        *coords - self.world_rect.point()
    }

    pub fn to_world_coords(&self, coords: &Vec2) -> Vec2 {
        *coords + self.world_rect.point()
    }

    pub fn draw(&self, bounding_rect: &Rect) {
        let extents = self.get_bounding_cell_rect_in_grid(&bounding_rect);
        for y in extents.top() as i64..extents.bottom() as i64 {
            for x in extents.left() as i64..extents.right() as i64 {
                if self.colliders[self.get_index(x, y)] == ColliderType::Solid {
                    draw_rectangle(
                        x as f32 * self.grid_size,
                        y as f32 * self.grid_size,
                        self.grid_size,
                        self.grid_size,
                        DARKGRAY,
                    )
                }
            }
        }
    }

    fn get_index(&self, x: i64, y: i64) -> usize {
        (y * self.width + x) as usize
    }

    fn is_occupied_at(&self, x: i64, y: i64) -> bool {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return false;
        }
        self.colliders[self.get_index(x, y)] != ColliderType::Empty
    }

    fn get_bounding_cell_rect_in_grid(&self, rect: &Rect) -> Rect {
        let max_x = (self.width) as f32;
        let max_y = (self.height) as f32;
        let left = clamp((rect.left() / self.grid_size).floor(), 0., max_x);
        let top = clamp((rect.top() / self.grid_size).floor(), 0., max_y);
        let right = clamp((rect.right() / self.grid_size).ceil(), 0., max_x);
        let bottom = clamp((rect.bottom() / self.grid_size).ceil(), 0., max_y);
        Rect::new(left, top, right - left, bottom - top)
    }

    pub fn get_bounding_cell_rect(&self, rect: &Rect) -> Rect {
        let mut result = self.get_bounding_cell_rect_in_grid(&rect);
        result.x *= self.grid_size;
        result.y *= self.grid_size;
        result.scale(self.grid_size, self.grid_size);
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
                let collider = Collider {
                    enable_top: !self.level.is_occupied_at(x, y - 1),
                    enable_bottom: !self.level.is_occupied_at(x, y + 1),
                    enable_left: !self.level.is_occupied_at(x - 1, y),
                    enable_right: !self.level.is_occupied_at(x + 1, y),
                    rect: Rect::new(
                        x as f32 * self.level.grid_size,
                        y as f32 * self.level.grid_size,
                        self.level.grid_size,
                        self.level.grid_size,
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
