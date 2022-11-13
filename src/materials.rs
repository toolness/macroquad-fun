use anyhow::Result;
use macroquad::{
    prelude::{
        gl_use_default_material, gl_use_material, load_material, load_string, Color, Material,
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

const LERP_TYPE_NONE: i32 = 0;
const LERP_TYPE_REPLACED_COLOR: i32 = 1;
const LERP_TYPE_ALL_COLORS: i32 = 2;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum LerpType {
    ReplacedColor,
    AllColors,
}

pub struct GameMaterials {
    pub replace_color_material: Material,
}

#[derive(Default, Clone, Copy)]
pub struct ReplaceColorOptions {
    pub image: Option<(&'static Image, f32)>,
    pub lerp: Option<(LerpType, Color, f32)>,
}

#[derive(Default, Clone, Copy)]
pub enum MaterialRenderer {
    #[default]
    None,
    ReplaceColors(ReplaceColorOptions),
}

impl MaterialRenderer {
    pub fn start_using(&self) {
        match self {
            MaterialRenderer::None => {}
            MaterialRenderer::ReplaceColors(options) => {
                use_replace_color_material(options);
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

pub fn replace_colors_with_image(image: &'static Image) -> MaterialRenderer {
    MaterialRenderer::ReplaceColors(ReplaceColorOptions {
        image: Some((image, 1.0)),
        ..Default::default()
    })
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
fn use_replace_color_material(options: &ReplaceColorOptions) {
    let materials = &game_assets().materials;
    let material = materials.replace_color_material;
    gl_use_material(material);

    match options.lerp {
        None => {
            material.set_uniform("lerp_type", LERP_TYPE_NONE);
        }
        Some((lerp_type, color, amount)) => {
            match lerp_type {
                LerpType::ReplacedColor => {
                    material.set_uniform("lerp_type", LERP_TYPE_REPLACED_COLOR)
                }
                LerpType::AllColors => material.set_uniform("lerp_type", LERP_TYPE_ALL_COLORS),
            }
            material.set_uniform("lerp_color", color.to_vec());
            material.set_uniform("lerp_amount", amount);
        }
    }

    let Some((image, find_replace_lerp_amount)) = options.image else {
        material.set_uniform("num_replacements", 0);
        return;
    };

    let num_replacements = (image.width / 2) as i32;

    material.set_uniform("num_replacements", num_replacements);
    material.set_uniform("find_replace_lerp_amount", find_replace_lerp_amount);

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
                            material.set_uniform("find_color_7", image.get_pixel(12, 0).to_vec());
                            material
                                .set_uniform("replace_color_7", image.get_pixel(13, 0).to_vec());
                            if num_replacements > 7 {
                                material
                                    .set_uniform("find_color_8", image.get_pixel(14, 0).to_vec());
                                material.set_uniform(
                                    "replace_color_8",
                                    image.get_pixel(15, 0).to_vec(),
                                );
                                if num_replacements > 8 {
                                    println!("Replacement color image has more than current maximum of 8 replacements!");
                                }
                            }
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
                    ("find_replace_lerp_amount".to_string(), UniformType::Float1),
                    ("find_color_1".to_string(), UniformType::Float4),
                    ("find_color_2".to_string(), UniformType::Float4),
                    ("find_color_3".to_string(), UniformType::Float4),
                    ("find_color_4".to_string(), UniformType::Float4),
                    ("find_color_5".to_string(), UniformType::Float4),
                    ("find_color_6".to_string(), UniformType::Float4),
                    ("find_color_7".to_string(), UniformType::Float4),
                    ("find_color_8".to_string(), UniformType::Float4),
                    ("replace_color_1".to_string(), UniformType::Float4),
                    ("replace_color_2".to_string(), UniformType::Float4),
                    ("replace_color_3".to_string(), UniformType::Float4),
                    ("replace_color_4".to_string(), UniformType::Float4),
                    ("replace_color_5".to_string(), UniformType::Float4),
                    ("replace_color_6".to_string(), UniformType::Float4),
                    ("replace_color_7".to_string(), UniformType::Float4),
                    ("replace_color_8".to_string(), UniformType::Float4),
                    ("lerp_type".to_string(), UniformType::Int1),
                    ("lerp_color".to_string(), UniformType::Float4),
                    ("lerp_amount".to_string(), UniformType::Float1),
                ],
                ..Default::default()
            },
        )
        .await?,
    })
}
