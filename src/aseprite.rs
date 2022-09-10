use std::collections::HashMap;

use anyhow::Result;
use macroquad::prelude::*;

use crate::config::config;

extern crate serde_derive;

#[derive(Deserialize)]
pub struct Aseprite {
    pub meta: Meta,
}

#[derive(Deserialize)]
pub struct Meta {
    pub slices: Vec<Slice>,
}

#[derive(Deserialize)]
pub struct Slice {
    pub name: String,
    pub keys: Vec<SliceKey>,
}

#[derive(Deserialize)]
pub struct SliceKey {
    pub bounds: Bound,
}

#[derive(Deserialize)]
pub struct Bound {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub async fn load_aseprite_slices(path: &str) -> Result<HashMap<String, Rect>> {
    let json_string = load_string(&path).await?;
    let aseprite: Aseprite = serde_json::from_str(json_string.as_str())?;
    let mut slices = HashMap::with_capacity(aseprite.meta.slices.len());
    let scale = config().sprite_scale;

    for slice in aseprite.meta.slices {
        let name = slice.name;
        let bounds = &slice.keys[0].bounds;
        slices.insert(
            name,
            Rect::new(
                bounds.x * scale,
                bounds.y * scale,
                bounds.w * scale,
                bounds.h * scale,
            ),
        );
    }

    Ok(slices)
}
