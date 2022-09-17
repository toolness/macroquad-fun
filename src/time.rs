use macroquad::time::get_time;

use crate::{config::config, sprite_renderer::SpriteRenderer};

pub struct GameTime {
    pub now: f64,
    pub absolute_frame_number: u64,
    pub time_since_last_frame: f64,
}

impl GameTime {
    pub fn new() -> Self {
        let now = get_time();
        GameTime {
            now,
            absolute_frame_number: 0,
            time_since_last_frame: 0.,
        }
    }

    pub fn update(&mut self) {
        let last_frame_time = self.now;
        self.now = get_time();
        self.absolute_frame_number = (self.now * 1000.0 / config().ms_per_animation_frame) as u64;
        self.time_since_last_frame = self.now - last_frame_time;
    }

    pub fn looping_frame_number(&self, sprite: &SpriteRenderer) -> u32 {
        (self.absolute_frame_number % sprite.num_frames() as u64) as u32
    }
}
