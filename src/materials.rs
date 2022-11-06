use anyhow::Result;
use macroquad::prelude::{
    gl_use_default_material, gl_use_material, load_material, load_string, Material, MaterialParams,
    UniformType,
};

use crate::{
    game_assets::game_assets,
    hex_color::{hex_color, HexColor},
};

const BASE_SHADER_PATH: &str = "media/shaders";

const LUIZ_MELO_RED: HexColor = hex_color("ff1831");

const BLACK: HexColor = hex_color("000000");

const WHITE: HexColor = hex_color("fbe9d1");

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
    RedToBlack,
    RedToWhite,
}

impl MaterialRenderer {
    pub fn start_using(&self) {
        match self {
            MaterialRenderer::None => {}
            MaterialRenderer::RedToBlack => {
                use_replace_color_material(LUIZ_MELO_RED, BLACK);
            }
            MaterialRenderer::RedToWhite => {
                use_replace_color_material(LUIZ_MELO_RED, WHITE);
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

fn use_replace_color_material(find_color: HexColor, replace_color: HexColor) {
    let materials = &game_assets().materials;
    let material = materials.replace_color_material;
    gl_use_material(material);
    material.set_uniform("find_color", find_color.vec3());
    material.set_uniform("replace_color", replace_color.vec3());
}

pub async fn load_game_materials() -> Result<GameMaterials> {
    Ok(GameMaterials {
        replace_color_material: load_shader(
            "replace_color",
            MaterialParams {
                uniforms: vec![
                    ("find_color".to_string(), UniformType::Float3),
                    ("replace_color".to_string(), UniformType::Float3),
                ],
                ..Default::default()
            },
        )
        .await?,
    })
}
