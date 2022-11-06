use anyhow::Result;
use macroquad::prelude::{load_material, load_string, vec3, Material, MaterialParams, UniformType};

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

pub fn red_to_blue(material: Material) {
    material.set_uniform("find_color", vec3(255., 24., 49.) / 255.);
    material.set_uniform("replace_color", vec3(0., 0., 255.) / 255.);
}
