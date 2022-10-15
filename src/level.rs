use anyhow::{anyhow, Error, Result};
use macroquad::prelude::*;

use crate::{
    collision::{Collider, CollisionFlags},
    config::config,
    game_sprites::game_sprites,
    ldtk::{self, field_into, EntityRef, LayerInstance, TileInstance},
    xy_range_iterator::XYRangeIterator,
};

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

    /// Foreground tiles for each grid cell, in row-major order. Corresponds to a
    /// Tiles layer in LDtk.
    pub tiles: Vec<Option<Tile>>,

    /// Background tiles for each grid cell, in row-major order. Corresponds to a
    /// Tiles layer in LDtk.
    pub background_tiles: Vec<Option<Tile>>,

    /// Tiles to use to draw the contents of entities, based on the entities' starting
    /// positions, for each grid cell, in row-major order. Corresponds to a Tiles layer
    /// in LDtk.
    pub entity_tiles: Vec<Option<Tile>>,

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

    /// The entity's Instance Identifier (from LDtk).
    pub iid: String,
}

#[derive(PartialEq)]
pub struct MovingPlatformArgs {
    pub end_point: Vec2,
    pub ping_pong: bool,
    pub stop_when_blocked: bool,
}

#[derive(PartialEq)]
pub enum EntityKind {
    PlayerStart(String),
    Text(Vec<String>),
    FlyingEye(Vec2),
    Mushroom,
    MovingPlatform(MovingPlatformArgs),
    Crate,
    FloorSwitch(Option<EntityRef>),
}

impl Level {
    pub fn from_ldtk(level: ldtk::Level) -> Result<Self> {
        let mut colliders: Option<Vec<ColliderType>> = None;
        let scale = config().sprite_scale;
        let world_rect = Rect::new(
            level.world_x as f32 * scale,
            level.world_y as f32 * scale,
            level.px_wid as f32 * scale,
            level.px_hei as f32 * scale,
        );
        let layers = level.layer_instances.unwrap();
        let mut entities = vec![];
        let mut opt_tiles: Option<Vec<Option<Tile>>> = None;
        let mut opt_background_tiles: Option<Vec<Option<Tile>>> = None;
        let mut opt_entity_tiles: Option<Vec<Option<Tile>>> = None;
        let first_layer = &layers
            .get(0)
            .expect("Level should have at least one layer!");
        let width: i64 = first_layer.c_wid;
        let height: i64 = first_layer.c_hei;
        let unscaled_grid_size: i64 = first_layer.grid_size;
        let grid_size: f32 = first_layer.grid_size as f32 * scale;
        for layer in layers {
            if layer.identifier == "IntGrid" {
                colliders = Some(ColliderType::from_vec(&layer.int_grid_csv)?);
                opt_tiles = Some(load_tile_layer(&layer, &layer.auto_layer_tiles));
            } else if layer.identifier == "Entities" {
                for entity in layer.entity_instances {
                    let rect = Rect::new(
                        entity.px[0] as f32 * scale,
                        entity.px[1] as f32 * scale,
                        entity.width as f32 * scale,
                        entity.height as f32 * scale,
                    );
                    let iid = entity.iid;
                    let mut fields = entity.field_instances;
                    let kind = match entity.identifier.as_str() {
                        "PlayerStart" => EntityKind::PlayerStart(field_into(&mut fields, "name")?),
                        "Text" => {
                            let text: String = field_into(&mut fields, "text")?;
                            let lines: Vec<String> =
                                text.split('\n').map(|s| s.to_owned()).collect();
                            EntityKind::Text(lines)
                        }
                        "FlyingEye" => EntityKind::FlyingEye(Vec2::new(
                            field_into(&mut fields, "x_velocity")?,
                            field_into(&mut fields, "y_velocity")?,
                        )),
                        "Mushroom" => EntityKind::Mushroom,
                        "MovingPlatform" => {
                            let end_point: Vec2 = field_into(&mut fields, "endpoint")?;
                            let ping_pong: bool = field_into(&mut fields, "ping_pong")?;
                            let stop_when_blocked: bool =
                                field_into(&mut fields, "stop_when_blocked")?;
                            EntityKind::MovingPlatform(MovingPlatformArgs {
                                end_point: end_point * grid_size,
                                ping_pong,
                                stop_when_blocked,
                            })
                        }
                        "Crate" => EntityKind::Crate,
                        "FloorSwitch" => {
                            let entity_ref: Option<EntityRef> = field_into(&mut fields, "trigger")?;
                            EntityKind::FloorSwitch(entity_ref)
                        }
                        _ => {
                            eprintln!("Unexpected entity found: {}", entity.identifier);
                            continue;
                        }
                    };
                    entities.push(Entity { kind, rect, iid });
                }
            } else if layer.identifier == "BackgroundTiles" {
                opt_background_tiles = Some(load_tile_layer(&layer, &layer.grid_tiles));
            } else if layer.identifier == "EntityTiles" {
                opt_entity_tiles = Some(load_tile_layer(&layer, &layer.grid_tiles));
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
            background_tiles: opt_background_tiles
                .ok_or(anyhow!("Couldn't find background tiles"))?,
            entity_tiles: opt_entity_tiles.ok_or(anyhow!("Couldn't find entity titles"))?,
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

    fn draw_tiles(
        &self,
        tiles: &Vec<Option<Tile>>,
        tileset: Texture2D,
        bounding_rect: &Rect,
        offset: &Vec2,
    ) {
        let tileset_rect = Rect {
            x: 0.,
            y: 0.,
            w: self.unscaled_grid_size as f32,
            h: self.unscaled_grid_size as f32,
        };
        let scaled_tile_size = Vec2::new(self.grid_size, self.grid_size);
        let extents: XYRangeIterator = self.get_bounding_cell_rect_in_grid(&bounding_rect).into();
        for (x, y) in extents {
            if let Some(tile) = self.get_tile_at(tiles, x, y) {
                draw_texture_ex(
                    tileset,
                    x as f32 * self.grid_size + offset.x,
                    y as f32 * self.grid_size + offset.y,
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

    pub fn draw_entity_tiles(&self, bounding_rect: &Rect, point: &Vec2) {
        let tileset = game_sprites().tileset;
        // We're using floor() here to avoid weird visual artifacts between tiles.
        let final_point = (*point).floor() - bounding_rect.point();
        self.draw_tiles(&self.entity_tiles, tileset, &bounding_rect, &final_point);
    }

    pub fn draw(&self, bounding_rect: &Rect) {
        let tileset = game_sprites().tileset;
        self.draw_tiles(&self.background_tiles, tileset, &bounding_rect, &Vec2::ZERO);
        self.draw_tiles(&self.tiles, tileset, &bounding_rect, &Vec2::ZERO);
    }

    fn get_index(&self, x: i64, y: i64) -> usize {
        (y * self.width + x) as usize
    }

    fn is_grid_coordinate_outside_of_bounds(&self, x: i64, y: i64) -> bool {
        x < 0 || x >= self.width || y < 0 || y >= self.height
    }

    fn get_tile_at(&self, tiles: &Vec<Option<Tile>>, x: i64, y: i64) -> Option<Tile> {
        if self.is_grid_coordinate_outside_of_bounds(x, y) {
            // Our code should be written in a way that we're preferably
            // never passed a tile that's out of bounds, but just in case
            // we do...
            println!("Warning: get_tile_at({}, {}) is out of bounds.", x, y);
            None
        } else {
            tiles[self.get_index(x, y)]
        }
    }

    fn is_occupied_at(&self, x: i64, y: i64) -> bool {
        if self.is_grid_coordinate_outside_of_bounds(x, y) {
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

    pub fn iter_colliders_ex<'a>(
        &'a self,
        bounding_rect: &Rect,
        include_bounds: bool,
    ) -> impl Iterator<Item = Collider> + 'a {
        let base = self.iter_colliders(bounding_rect);

        if include_bounds {
            base.chain(self.iter_bounds_as_colliders())
        } else {
            base.chain(BoundsColliderIterator::empty())
        }
    }

    pub fn iter_colliders(&self, bounding_rect: &Rect) -> GridColliderIterator {
        let extents = self.get_bounding_cell_rect_in_grid(&bounding_rect);
        GridColliderIterator {
            level: &self,
            range: extents.into(),
        }
    }

    pub fn is_area_vacant(&self, bounding_rect: &Rect) -> bool {
        for collider in self.iter_colliders(bounding_rect) {
            if collider.rect.overlaps(bounding_rect) {
                return false;
            }
        }
        true
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

fn load_tile_layer(layer: &LayerInstance, layer_tiles: &Vec<TileInstance>) -> Vec<Option<Tile>> {
    let mut tiles: Vec<Option<Tile>> = vec![None; layer.c_wid as usize * layer.c_hei as usize];
    for grid_tile in layer_tiles.iter() {
        let grid_x = grid_tile.layer_px[0] / layer.grid_size;
        let grid_y = grid_tile.layer_px[1] / layer.grid_size;
        let tileset_px = Vec2::new(
            grid_tile.tileset_px[0] as f32,
            grid_tile.tileset_px[1] as f32,
        );
        let index = (grid_y * layer.c_wid + grid_x) as usize;
        tiles[index] = Some(Tile { tileset_px });
    }
    tiles
}

pub struct BoundsColliderIterator {
    bounds: Rect,
    position: u8,
}

impl BoundsColliderIterator {
    pub fn empty() -> Self {
        BoundsColliderIterator {
            bounds: Default::default(),
            position: 4,
        }
    }
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
    range: XYRangeIterator,
}

impl<'a> Iterator for GridColliderIterator<'a> {
    type Item = Collider;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((x, y)) = self.range.next() {
                if self.level.is_occupied_at(x, y) {
                    let rect = Rect::new(
                        x as f32 * self.level.grid_size,
                        y as f32 * self.level.grid_size,
                        self.level.grid_size,
                        self.level.grid_size,
                    );
                    let collider = Collider {
                        enable_top: !self.level.is_occupied_at(x, y - 1),
                        enable_bottom: !self.level.is_occupied_at(x, y + 1),
                        enable_left: !self.level.is_occupied_at(x - 1, y),
                        enable_right: !self.level.is_occupied_at(x + 1, y),
                        flags: CollisionFlags::ENVIRONMENT,
                        rect,
                        entity_id: None,
                        prev_rect: rect,
                        velocity: Vec2::ZERO,
                    };
                    return Some(collider);
                } else {
                    continue;
                }
            } else {
                return None;
            }
        }
    }
}
