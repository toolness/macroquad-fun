#[cfg(target_arch = "wasm32")]
use crate::js_interop::js_interop_wasm32 as js;

use macroquad::{
    audio::{load_sound, play_sound, PlaySoundParams, Sound},
    prelude::FileError,
};

pub type SoundEffect = Option<Sound>;

pub async fn load_sound_effect(path: &'static str) -> Result<SoundEffect, FileError> {
    #[cfg(target_arch = "wasm32")]
    {
        if !js::is_ogg_supported() {
            let mp3_path = path.replace(".ogg", ".mp3");
            if let Ok(sound) = load_sound(&mp3_path).await {
                return Ok(Some(sound));
            }
            return Ok(None);
        }
    }

    Ok(Some(load_sound(path).await?))
}

pub fn play_sound_effect_at_volume(sound_effect: SoundEffect, volume: f32) {
    if let Some(sound) = sound_effect {
        play_sound(
            sound,
            PlaySoundParams {
                volume,
                ..Default::default()
            },
        );
    }
}

pub fn play_sound_effect(sound_effect: SoundEffect) {
    play_sound_effect_at_volume(sound_effect, 0.25);
}
