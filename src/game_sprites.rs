use std::collections::HashMap;

use anyhow::{anyhow, Result};
use macroquad::{prelude::Rect, texture::load_texture};

use crate::{aseprite::load_aseprite_slices, sprite::Sprite};

pub struct HuntressSprites {
    pub idle: Sprite,
    pub run: Sprite,
    pub jump: Sprite,
    pub fall: Sprite,
    pub idle_bbox: Rect,
}

pub struct FlyingEyeSprites {
    pub flight: Sprite,
    pub flight_bbox: Rect,
}

pub struct GameSprites {
    pub huntress: HuntressSprites,
    pub flying_eye: FlyingEyeSprites,
}

fn get_slice(slices: &HashMap<String, Rect>, name: &str) -> Result<Rect> {
    if let Some(&slice) = slices.get(name) {
        Ok(slice)
    } else {
        Err(anyhow!("Slice not found: '{}'", name))
    }
}

pub async fn load_game_sprites(scale: f32) -> Result<GameSprites> {
    let sprites = GameSprites {
        huntress: HuntressSprites {
            idle: Sprite::new(load_texture("media/Huntress/Idle.png").await?, 8, scale),
            run: Sprite::new(load_texture("media/Huntress/Run.png").await?, 8, scale),
            jump: Sprite::new(load_texture("media/Huntress/Jump.png").await?, 2, scale),
            fall: Sprite::new(load_texture("media/Huntress/Fall.png").await?, 2, scale),
            idle_bbox: get_slice(
                &load_aseprite_slices("media/Huntress/Idle.json", scale).await?,
                "idle_bounding_box",
            )?,
        },
        flying_eye: FlyingEyeSprites {
            flight: Sprite::new(load_texture("media/FlyingEye/Flight.png").await?, 8, scale),
            flight_bbox: get_slice(
                &load_aseprite_slices("media/FlyingEye/Flight.json", scale).await?,
                "flight_bounding_box",
            )?,
        },
    };

    Ok(sprites)
}
