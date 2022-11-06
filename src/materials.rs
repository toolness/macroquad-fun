use anyhow::Result;
use macroquad::{
    prelude::{
        gl_use_default_material, gl_use_material, load_material, load_string, Material,
        MaterialParams,
    },
    texture::Texture2D,
};

use crate::game_assets::game_assets;

const BASE_SHADER_PATH: &str = "media/shaders";

async fn load_shader(stem: &str, params: MaterialParams) -> Result<Material> {
    let vertex_source = load_string(format!("{}/{}.vert", BASE_SHADER_PATH, stem).as_str()).await?;
    let fragment_source =
        load_string(format!("{}/{}.frag", BASE_SHADER_PATH, stem).as_str()).await?;

    let material = load_material(vertex_source.as_str(), fragment_source.as_str(), params)?;

    Ok(material)
}

pub struct GameMaterials {
    pub replace_color_material: Material,
}

#[derive(Default, Clone, Copy)]
pub enum MaterialRenderer {
    #[default]
    None,
    ReplaceColor(Texture2D),
}

impl MaterialRenderer {
    pub fn start_using(&self) {
        match self {
            MaterialRenderer::None => {}
            MaterialRenderer::ReplaceColor(texture) => {
                use_replace_color_material(*texture);
            }
        }
    }

    pub fn stop_using(&self) {
        match self {
            MaterialRenderer::None => {}
            _ => {
                gl_use_default_material();
            }
        }
    }
}

fn use_replace_color_material(color_replacement_texture: Texture2D) {
    let materials = &game_assets().materials;
    let material = materials.replace_color_material;
    gl_use_material(material);
    material.set_texture("color_replacement_texture", color_replacement_texture);
}

pub async fn load_game_materials() -> Result<GameMaterials> {
    Ok(GameMaterials {
        replace_color_material: load_shader(
            "replace_color",
            MaterialParams {
                textures: vec!["color_replacement_texture".to_string()],
                ..Default::default()
            },
        )
        .await?,
    })
}
