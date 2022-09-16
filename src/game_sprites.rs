use std::collections::HashMap;

use anyhow::{anyhow, Result};
use macroquad::{
    prelude::Rect,
    texture::{load_texture, FilterMode, Texture2D},
};

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

pub struct MushroomSprites {
    pub death: Sprite,
    pub idle_bbox: Rect,
    pub run: Sprite,
}

pub struct GameSprites {
    pub huntress: HuntressSprites,
    pub flying_eye: FlyingEyeSprites,
    pub mushroom: MushroomSprites,
    pub tileset: Texture2D,
}

fn get_slice(slices: &HashMap<String, Rect>, name: &str) -> Result<Rect> {
    if let Some(&slice) = slices.get(name) {
        Ok(slice)
    } else {
        Err(anyhow!("Slice not found: '{}'", name))
    }
}

async fn load_pixel_perfect_texture(path: &str) -> Result<Texture2D> {
    let texture = load_texture(path).await?;
    texture.set_filter(FilterMode::Nearest);
    Ok(texture)
}

pub async fn load_game_sprites() -> Result<()> {
    let sprites = GameSprites {
        huntress: HuntressSprites {
            idle: Sprite::new(load_texture("media/Huntress/Idle.png").await?, 8),
            run: Sprite::new(load_texture("media/Huntress/Run.png").await?, 8),
            jump: Sprite::new(load_texture("media/Huntress/Jump.png").await?, 2),
            fall: Sprite::new(load_texture("media/Huntress/Fall.png").await?, 2),
            idle_bbox: get_slice(
                &load_aseprite_slices("media/Huntress/Idle.json").await?,
                "idle_bounding_box",
            )?,
        },
        flying_eye: FlyingEyeSprites {
            flight: Sprite::new(load_texture("media/FlyingEye/Flight.png").await?, 8),
            flight_bbox: get_slice(
                &load_aseprite_slices("media/FlyingEye/Flight.json").await?,
                "flight_bounding_box",
            )?,
        },
        mushroom: MushroomSprites {
            death: Sprite::new(load_texture("media/Mushroom/Death.png").await?, 4),
            idle_bbox: get_slice(
                &load_aseprite_slices("media/Mushroom/Idle.json").await?,
                "idle_bounding_box",
            )?,
            run: Sprite::new(load_texture("media/Mushroom/Run.png").await?, 8),
        },
        tileset: load_pixel_perfect_texture("media/bigbrick1.png").await?,
    };

    unsafe {
        GAME_SPRITES = Some(sprites);
    }

    Ok(())
}

pub fn game_sprites() -> &'static GameSprites {
    unsafe {
        GAME_SPRITES
            .as_ref()
            .expect("load_game_sprites() was not called or did not finish")
    }
}

static mut GAME_SPRITES: Option<GameSprites> = None;
