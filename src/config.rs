use anyhow::Result;
use macroquad::prelude::*;

extern crate serde_derive;

#[derive(Deserialize)]
pub struct Config {
    pub sprite_scale: f32,
    pub ms_per_animation_frame: f64,
    pub ms_to_max_run_speed: f64,
    pub run_speed: f32,
    pub gravity: f32,
    pub long_jump_keypress_extra_force: f32,
    pub jump_velocity: f32,
    pub flying_eye_speed: f32,
    pub mushroom_speed: f32,
    pub screen_width: f32,
    pub screen_height: f32,
    pub fall_off_level_threshold: f32,
}

pub async fn load_config(path: &str) -> Result<()> {
    let mut config: Config = serde_json::from_str(&load_string(path).await?)?;
    config.run_speed *= config.sprite_scale;
    config.gravity *= config.sprite_scale;
    config.jump_velocity *= config.sprite_scale;
    config.long_jump_keypress_extra_force *= config.sprite_scale;
    config.screen_width *= config.sprite_scale;
    config.screen_height *= config.sprite_scale;
    config.flying_eye_speed *= config.sprite_scale;
    config.mushroom_speed *= config.sprite_scale;
    config.fall_off_level_threshold *= config.sprite_scale;

    unsafe {
        CONFIG = Some(config);
    }

    Ok(())
}

pub fn config() -> &'static Config {
    unsafe {
        CONFIG
            .as_ref()
            .expect("load_config() was not called or did not finish")
    }
}

static mut CONFIG: Option<Config> = None;
