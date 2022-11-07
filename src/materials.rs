use anyhow::Result;
use macroquad::{
    prelude::{
        gl_use_default_material, gl_use_material, load_material, load_string, Material,
        MaterialParams, UniformType,
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

/// Use an image to specify what colors to replace at render time.
///
/// The image should be structured in such a way that each pixel on the x-axis
/// is immediately followed by the color that should replace it.
///
/// So for instance, if you have a 4x1 image that consists of a blue
/// pixel, a red pixel, a green pixel, and a yellow pixel, this means that
/// whenever this material is used, all blue pixels will be replaced by red
/// ones, and all green pixels will be replaced by yellow ones.
fn use_replace_color_material(image: &Image) {
    let materials = &game_assets().materials;
    let material = materials.replace_color_material;
    gl_use_material(material);
    let num_replacements = (image.width / 2) as i32;

    material.set_uniform("num_replacements", num_replacements);

    // Ideally we'd just use a Texture2D for this, allowing the GPU to do all this work
    // itself, and easily supporting an arbitrary number of replacements. However, there
    // seems to be a bug in macroquad, or my own misunderstanding of how it works, that
    // prevents it from working; see commit 643d19b627d5626d12e3affe567717bace37a247 or
    // https://github.com/toolness/macroquad-fun/pull/57 for more details on that attempt.
    //
    // So for now we'll just load the image on the CPU-side and set a bunch of uniforms
    // that tell the GPU what to replace.

    material.set_uniform("find_color_1", image.get_pixel(0, 0).to_vec());
    material.set_uniform("replace_color_1", image.get_pixel(1, 0).to_vec());

    if num_replacements > 1 {
        material.set_uniform("find_color_2", image.get_pixel(2, 0).to_vec());
        material.set_uniform("replace_color_2", image.get_pixel(3, 0).to_vec());
        if num_replacements > 2 {
            material.set_uniform("find_color_3", image.get_pixel(4, 0).to_vec());
            material.set_uniform("replace_color_3", image.get_pixel(5, 0).to_vec());
            if num_replacements > 3 {
                material.set_uniform("find_color_4", image.get_pixel(6, 0).to_vec());
                material.set_uniform("replace_color_4", image.get_pixel(7, 0).to_vec());
                if num_replacements > 4 {
                    material.set_uniform("find_color_5", image.get_pixel(8, 0).to_vec());
                    material.set_uniform("replace_color_5", image.get_pixel(9, 0).to_vec());
                    if num_replacements > 5 {
                        material.set_uniform("find_color_6", image.get_pixel(10, 0).to_vec());
                        material.set_uniform("replace_color_6", image.get_pixel(11, 0).to_vec());
                        if num_replacements > 6 {
                            println!("Replacement color image has more than current maximum of 6 replacements!");
                        }
                    }
                }
            }
        }
    }
}

pub async fn load_game_materials() -> Result<GameMaterials> {
    Ok(GameMaterials {
        replace_color_material: load_shader(
            "replace_color",
            MaterialParams {
                uniforms: vec![
                    ("num_replacements".to_string(), UniformType::Int1),
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
