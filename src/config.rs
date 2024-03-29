use anyhow::Result;
use macroquad::prelude::*;

use crate::cli::Cli;

extern crate serde_derive;

#[derive(Deserialize)]
pub struct Config {
    pub fixed_fps: u64,
    pub sprite_scale: f32,
    pub ms_per_animation_frame: f64,
    pub ms_per_text_char: f64,
    pub ms_to_max_run_speed: f64,
    pub run_speed: f32,
    pub gravity: f32,
    pub player_left_facing_x_offset: f32,
    pub long_jump_keypress_extra_force: f32,
    pub jump_velocity: f32,
    pub blocked_route_edge_thickness: f32,
    pub attach_velocity_coefficient: f32,
    pub flying_eye_speed: f32,
    pub moving_platform_speed: f32,
    pub mushroom_speed: f32,
    pub mushroom_rez_ms_per_animation_frame: f64,
    pub pickup_float_frequency: f32,
    pub pickup_float_amplitude: f32,
    pub screen_width: f32,
    pub screen_height: f32,
    pub crate_pushable_coefficient: f32,
    pub fall_off_level_threshold: f32,
    pub vertical_collision_leeway: f32,
    pub life_transfer_rate: f32,
    pub coyote_time_ms: f64,
    pub debug_text_size: f32,
}

pub fn parse_config(config: &str, args: &Cli) -> Result<Config> {
    let mut config: Config = serde_json::from_str(config)?;

    config.run_speed *= config.sprite_scale;
    config.gravity *= config.sprite_scale;
    config.jump_velocity *= config.sprite_scale;
    config.long_jump_keypress_extra_force *= config.sprite_scale;
    config.screen_width *= config.sprite_scale;
    config.screen_height *= config.sprite_scale;
    config.flying_eye_speed *= config.sprite_scale;
    config.mushroom_speed *= config.sprite_scale;
    config.fall_off_level_threshold *= config.sprite_scale;
    config.moving_platform_speed *= config.sprite_scale;
    config.vertical_collision_leeway *= config.sprite_scale;
    config.blocked_route_edge_thickness *= config.sprite_scale;
    config.player_left_facing_x_offset *= config.sprite_scale;
    config.pickup_float_amplitude *= config.sprite_scale;

    if let Some(width) = args.width {
        config.screen_width = width as f32;
    }

    if let Some(height) = args.height {
        config.screen_height = height as f32;
    }

    Ok(config)
}

pub async fn load_config(path: &str, args: &Cli) -> Result<()> {
    let config = parse_config(&load_string(path).await?, args)?;

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
