use macroquad::time::get_time;

use crate::{config::config, sprite_renderer::SpriteRenderer};

/// Our minimum FPS for timing purposes. Even if the system is really slow,
/// or hiccups for some reason, this is the minimum rate at which we'll
/// process each frame, to ensure that really weird things like tunneling
/// don't happen. This does mean that really slow devices will appear to
/// run in slow motion.
///
/// (Note that if we actually want to play the game in slow motion, we can
/// set this to a ridiculously high value!)
const MIN_FPS: f64 = 30.0;

const MAX_TIME_SINCE_LAST_FRAME: f64 = 1. / MIN_FPS;

pub struct GameTime {
    pub now: f64,
    pub absolute_frame_number: u64,
    pub time_since_last_frame: f64,
    excess_time_offset: f64,
}

impl GameTime {
    pub fn new() -> Self {
        let now = get_time();
        GameTime {
            now,
            absolute_frame_number: 0,
            time_since_last_frame: 0.,
            excess_time_offset: 0.,
        }
    }

    pub fn update(&mut self) {
        let last_frame_time = self.now;
        self.now = get_time() - self.excess_time_offset;
        self.time_since_last_frame = self.now - last_frame_time;
        if self.time_since_last_frame > MAX_TIME_SINCE_LAST_FRAME {
            self.time_since_last_frame = MAX_TIME_SINCE_LAST_FRAME;

            // Our "real" time as reported from macroquad is has now deviated from our
            // in-game concept of time, so adjust accordingly.
            let delta = self.time_since_last_frame - MAX_TIME_SINCE_LAST_FRAME;
            self.excess_time_offset += delta;
            self.now -= delta;
        }
        self.absolute_frame_number = (self.now * 1000.0 / config().ms_per_animation_frame) as u64;
    }

    pub fn looping_frame_number(&self, sprite: &SpriteRenderer) -> u32 {
        (self.absolute_frame_number % sprite.num_frames() as u64) as u32
    }
}
