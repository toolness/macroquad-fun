use macroquad::prelude::clamp;

use crate::{config::config, time::GameTime};

pub struct Animator {
    last_frame: u32,
    is_reversed: bool,
    start_time: f64,
    ms_per_animation_frame: f64,
}

impl Animator {
    pub fn new(last_frame: u32, is_reversed: bool, time: &GameTime) -> Self {
        Animator {
            last_frame,
            is_reversed,
            start_time: time.now,
            ms_per_animation_frame: config().ms_per_animation_frame,
        }
    }

    fn get_unclamped_frame(&self, time: &GameTime) -> u32 {
        let time_since_start = time.now - self.start_time;
        (time_since_start * 1000.0 / self.ms_per_animation_frame) as u32
    }

    pub fn is_done(&self, time: &GameTime) -> bool {
        self.get_unclamped_frame(&time) > self.last_frame
    }

    pub fn get_frame(&self, time: &GameTime) -> u32 {
        let frame = clamp(self.get_unclamped_frame(&time), 0, self.last_frame);
        if self.is_reversed {
            self.last_frame - frame
        } else {
            frame
        }
    }
}
