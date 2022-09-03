use anyhow::Result;
use macroquad::prelude::*;

extern crate serde_derive;

#[derive(Deserialize)]
pub struct Config {
    pub sprite_scale: f32,
    pub ms_per_animation_frame: f64,
    pub run_speed: f64,
    pub gravity: f32,
    pub jump_velocity: f32,
}

pub async fn load_config(path: &str) -> Result<Config> {
    let config: Config = serde_json::from_str(&load_string(path).await?)?;
    Ok(config)
}
