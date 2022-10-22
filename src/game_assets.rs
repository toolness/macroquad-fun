use std::collections::HashMap;

use anyhow::{anyhow, Result};
use macroquad::{
    prelude::{load_material, load_string, Material, Rect},
    texture::{load_texture, FilterMode, Texture2D},
};

use crate::{aseprite::load_aseprite_slices, font::BitmapFont, sprite_renderer::SpriteRenderer};

pub struct HuntressAssets {
    pub idle: SpriteRenderer,
    pub run: SpriteRenderer,
    pub jump: SpriteRenderer,
    pub fall: SpriteRenderer,
    pub idle_bbox: Rect,
}

pub struct FlyingEyeAssets {
    pub flight: SpriteRenderer,
    pub flight_bbox: Rect,
}

pub struct MushroomAssets {
    pub death: SpriteRenderer,
    pub idle_bbox: Rect,
    pub platform_bbox: Rect,
    pub run: SpriteRenderer,
}

pub struct GameAssets {
    pub huntress: HuntressAssets,
    pub flying_eye: FlyingEyeAssets,
    pub mushroom: MushroomAssets,
    pub tileset: Texture2D,
    pub font: BitmapFont,
    pub crt_material: Material,
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

const BASE_SHADER_PATH: &str = "media/shaders";

async fn load_shader(stem: &str) -> Result<Material> {
    let vertex_source = load_string(format!("{}/{}.vert", BASE_SHADER_PATH, stem).as_str()).await?;
    let fragment_source =
        load_string(format!("{}/{}.frag", BASE_SHADER_PATH, stem).as_str()).await?;

    let material = load_material(
        vertex_source.as_str(),
        fragment_source.as_str(),
        Default::default(),
    )?;

    Ok(material)
}

pub async fn load_game_assets() -> Result<()> {
    let mushroom_idle_slices = load_aseprite_slices("media/Mushroom/Idle.json").await?;
    let assets = GameAssets {
        huntress: HuntressAssets {
            idle: SpriteRenderer::new(load_texture("media/Huntress/Idle.png").await?, 8),
            run: SpriteRenderer::new(load_texture("media/Huntress/Run.png").await?, 8),
            jump: SpriteRenderer::new(load_texture("media/Huntress/Jump.png").await?, 2),
            fall: SpriteRenderer::new(load_texture("media/Huntress/Fall.png").await?, 2),
            idle_bbox: get_slice(
                &load_aseprite_slices("media/Huntress/Idle.json").await?,
                "idle_bounding_box",
            )?,
        },
        flying_eye: FlyingEyeAssets {
            flight: SpriteRenderer::new(load_texture("media/FlyingEye/Flight.png").await?, 8),
            flight_bbox: get_slice(
                &load_aseprite_slices("media/FlyingEye/Flight.json").await?,
                "flight_bounding_box",
            )?,
        },
        mushroom: MushroomAssets {
            death: SpriteRenderer::new(load_texture("media/Mushroom/Death.png").await?, 4),
            idle_bbox: get_slice(&mushroom_idle_slices, "idle_bounding_box")?,
            platform_bbox: get_slice(&mushroom_idle_slices, "platform_bounding_box")?,
            run: SpriteRenderer::new(load_texture("media/Mushroom/Run.png").await?, 8),
        },
        tileset: load_pixel_perfect_texture("media/bigbrick1.png").await?,
        font: BitmapFont {
            texture: load_pixel_perfect_texture("media/pman_font01.png").await?,
            char_width: 6,
            char_height: 8,
            chars_per_line: 16,
        },
        crt_material: load_shader("crt").await?,
    };

    unsafe {
        GAME_ASSETS = Some(assets);
    }

    Ok(())
}

pub fn game_assets() -> &'static GameAssets {
    unsafe {
        GAME_ASSETS
            .as_ref()
            .expect("load_game_assets() was not called or did not finish")
    }
}

static mut GAME_ASSETS: Option<GameAssets> = None;
