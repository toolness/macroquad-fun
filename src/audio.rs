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
            return Ok(None);
        }
    }

    Ok(Some(load_sound(path).await?))
}

pub fn play_sound_effect(sound_effect: SoundEffect) {
    if let Some(sound) = sound_effect {
        play_sound(
            sound,
            PlaySoundParams {
                volume: 0.25,
                ..Default::default()
            },
        );
    }
}
