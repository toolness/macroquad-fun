use anyhow::Result;
use macroquad::{
    prelude::{
        gl_use_default_material, gl_use_material, load_material, load_string, Material,
        MaterialParams, UniformType, Vec4,
    },
    texture::Image,
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
    ReplaceColors(&'static Image),
}

impl MaterialRenderer {
    pub fn start_using(&self) {
        match self {
            MaterialRenderer::None => {}
            MaterialRenderer::ReplaceColors(image) => {
                use_replace_color_material(image);
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

fn use_replace_color_material(image: &Image) {
    let materials = &game_assets().materials;
    let material = materials.replace_color_material;
    gl_use_material(material);

    material.set_uniform("find_color_1", image.get_pixel(0, 0).to_vec());
    material.set_uniform("replace_color_1", image.get_pixel(1, 0).to_vec());

    material.set_uniform("find_color_2", Vec4::ZERO);
    material.set_uniform("find_color_3", Vec4::ZERO);
    material.set_uniform("find_color_4", Vec4::ZERO);
    material.set_uniform("find_color_5", Vec4::ZERO);
    material.set_uniform("find_color_6", Vec4::ZERO);
    material.set_uniform("replace_color_2", Vec4::ZERO);
    material.set_uniform("replace_color_3", Vec4::ZERO);
    material.set_uniform("replace_color_4", Vec4::ZERO);
    material.set_uniform("replace_color_5", Vec4::ZERO);
    material.set_uniform("replace_color_6", Vec4::ZERO);
}

pub async fn load_game_materials() -> Result<GameMaterials> {
    Ok(GameMaterials {
        replace_color_material: load_shader(
            "replace_color",
            MaterialParams {
                uniforms: vec![
                    ("find_color_1".to_string(), UniformType::Float4),
                    ("find_color_2".to_string(), UniformType::Float4),
                    ("find_color_3".to_string(), UniformType::Float4),
                    ("find_color_4".to_string(), UniformType::Float4),
                    ("find_color_5".to_string(), UniformType::Float4),
                    ("find_color_6".to_string(), UniformType::Float4),
                    ("replace_color_1".to_string(), UniformType::Float4),
                    ("replace_color_2".to_string(), UniformType::Float4),
                    ("replace_color_3".to_string(), UniformType::Float4),
                    ("replace_color_4".to_string(), UniformType::Float4),
                    ("replace_color_5".to_string(), UniformType::Float4),
                    ("replace_color_6".to_string(), UniformType::Float4),
                ],
                ..Default::default()
            },
        )
        .await?,
    })
}
