use macroquad::audio::{play_sound, PlaySoundParams, Sound};

pub fn play_sound_effect(sound: Sound) {
    play_sound(
        sound,
        PlaySoundParams {
            volume: 0.25,
            ..Default::default()
        },
    );
}
