use anyhow::{anyhow, Error, Result};
use macroquad::prelude::*;

use crate::{collision::Collider, config::config, game_sprites::game_sprites, ldtk};

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

pub struct Level {
    /// Unique name for the level.
    pub identifier: String,

    /// Width in grid cells.
    pub width: i64,

    /// Height in grid cells.
    pub height: i64,

    /// Width/height of each grid cell in pixels, scaled.
    pub grid_size: f32,

    /// Width/height of each grid cell in pixels, unscaled.
    pub unscaled_grid_size: i64,

    /// Where the level exists in world coordinates.
    pub world_rect: Rect,

    /// Colliders for each grid cell, in row-major order. Corresponds to
    /// an IntGrid layer in LDtk.
    pub colliders: Vec<ColliderType>,

    /// Tiles for each grid cell, in row-major order. Corresponds to a
    /// Tiles layer in LDtk.
    pub tiles: Vec<Option<Tile>>,

    /// Various other entities in the level.
    pub entities: Vec<Entity>,
}

#[derive(Copy, Clone)]
pub struct Tile {
    /// The top-left corner of the tile to use from the tileset, in pixels.
    pub tileset_px: Vec2,
}

pub struct Entity {
    /// The type of entity.
    pub kind: EntityKind,

    /// The entity's position in pixel coordinates.
    pub rect: Rect,
}

#[derive(PartialEq)]
pub enum EntityKind {
    PlayerStart,
    Text(Vec<String>),
    FlyingEye(Vec2),
    Mushroom,
}

impl Level {
    pub fn from_ldtk(level: &ldtk::Level) -> Result<Self> {
        let mut colliders: Option<Vec<ColliderType>> = None;
        let mut width: i64 = 0;
        let mut height: i64 = 0;
        let mut unscaled_grid_size: i64 = 0;
        let mut grid_size: f32 = 0.;
        let scale = config().sprite_scale;
        let world_rect = Rect::new(
            level.world_x as f32 * scale,
            level.world_y as f32 * scale,
            level.px_wid as f32 * scale,
            level.px_hei as f32 * scale,
        );
        let layers = level.layer_instances.as_ref().unwrap();
        let mut entities = vec![];
        let mut opt_tiles: Option<Vec<Option<Tile>>> = None;
        for layer in layers.iter() {
            if layer.identifier == "IntGrid" {
                width = layer.c_wid;
                height = layer.c_hei;
                unscaled_grid_size = layer.grid_size;
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
                    let kind = match entity.identifier.as_str() {
                        "PlayerStart" => EntityKind::PlayerStart,
                        "Text" => {
                            let lines: Vec<String> = entity
                                .get_string_field_instance("text")?
                                .split('\n')
                                .map(|s| s.to_owned())
                                .collect();
                            EntityKind::Text(lines)
                        }
                        "FlyingEye" => EntityKind::FlyingEye(Vec2::new(
                            entity.get_float_field_instance("x_velocity")? as f32,
                            entity.get_float_field_instance("y_velocity")? as f32,
                        )),
                        "Mushroom" => EntityKind::Mushroom,
                        _ => {
                            eprintln!("Unexpected entity found: {}", entity.identifier);
                            continue;
                        }
                    };
                    entities.push(Entity { kind, rect });
                }
            } else if layer.identifier == "Tiles" {
                let mut tiles: Vec<Option<Tile>> =
                    vec![None; layer.c_wid as usize * layer.c_hei as usize];
                for grid_tile in layer.grid_tiles.iter() {
                    let grid_x = grid_tile.layer_px[0] / layer.grid_size;
                    let grid_y = grid_tile.layer_px[1] / layer.grid_size;
                    let tileset_px = Vec2::new(
                        grid_tile.tileset_px[0] as f32,
                        grid_tile.tileset_px[1] as f32,
                    );
                    let index = (grid_y * layer.c_wid + grid_x) as usize;
                    tiles[index] = Some(Tile { tileset_px });
                }
                opt_tiles = Some(tiles);
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
            unscaled_grid_size,
            colliders: colliders.ok_or(anyhow!("Couldn't find colliders"))?,
            tiles: opt_tiles.ok_or(anyhow!("Couldn't find tiles"))?,
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
        let tileset = game_sprites().tileset;
        let tileset_rect = Rect {
            x: 0.,
            y: 0.,
            w: self.unscaled_grid_size as f32,
            h: self.unscaled_grid_size as f32,
        };
        let scaled_tile_size = Vec2::new(self.grid_size, self.grid_size);
        let extents = self.get_bounding_cell_rect_in_grid(&bounding_rect);
        for y in extents.top() as i64..extents.bottom() as i64 {
            for x in extents.left() as i64..extents.right() as i64 {
                if let Some(tile) = self.tiles[self.get_index(x, y)] {
                    draw_texture_ex(
                        tileset,
                        x as f32 * self.grid_size,
                        y as f32 * self.grid_size,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(scaled_tile_size),
                            source: Some(tileset_rect.offset(tile.tileset_px)),
                            ..Default::default()
                        },
                    );
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

    pub fn iter_bounds_as_colliders(&self) -> BoundsColliderIterator {
        BoundsColliderIterator {
            bounds: self.pixel_bounds(),
            position: 0,
        }
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

    pub fn get_text(&self, rect: &Rect) -> Option<&Vec<String>> {
        for entity in self.entities.iter() {
            if let EntityKind::Text(text) = &entity.kind {
                if entity.rect.overlaps(rect) {
                    return Some(text);
                }
            }
        }
        None
    }
}

pub struct BoundsColliderIterator {
    bounds: Rect,
    position: u8,
}

impl Iterator for BoundsColliderIterator {
    type Item = Collider;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.position {
            0 => Collider {
                // Top side of level.
                rect: self.bounds.offset(Vec2::new(0., -self.bounds.h)),
                enable_bottom: true,
                ..Default::default()
            },
            1 => Collider {
                // Right side of level.
                rect: self.bounds.offset(Vec2::new(self.bounds.w, 0.)),
                enable_left: true,
                ..Default::default()
            },
            2 => Collider {
                // Bottom side of level.
                rect: self.bounds.offset(Vec2::new(0., self.bounds.h)),
                enable_top: true,
                ..Default::default()
            },
            3 => Collider {
                // Left side of level.
                rect: self.bounds.offset(Vec2::new(-self.bounds.w, 0.)),
                enable_right: true,
                ..Default::default()
            },
            _ => {
                return None;
            }
        };
        self.position += 1;
        Some(result)
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
