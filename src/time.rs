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
    pub fn new(now: f64) -> Self {
        GameTime {
            now,
            absolute_frame_number: 0,
            time_since_last_frame: 0.,
            excess_time_offset: 0.,
        }
    }

    pub fn update(&mut self, now: f64) {
        let last_frame_time = self.now;
        self.now = now - self.excess_time_offset;
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

pub struct FixedGameTime {
    now: f64,
    frames_so_far: u64,
    start: f64,
    frame_duration: f64,
}

impl FixedGameTime {
    pub fn new(fixed_frame_rate: u64, now: f64) -> Self {
        FixedGameTime {
            now,
            start: now,
            frames_so_far: 0,
            frame_duration: 1. / (fixed_frame_rate as f64),
        }
    }

    pub fn update(&mut self, now: f64) {
        self.now = now;
    }
}

impl Iterator for FixedGameTime {
    type Item = GameTime;

    fn next(&mut self) -> Option<Self::Item> {
        let time_passed = self.now - self.start;
        let total_frames = (time_passed / self.frame_duration) as u64;
        if total_frames > self.frames_so_far {
            self.frames_so_far += 1;
            let synthetic_now = self.start + (self.frames_so_far as f64) * self.frame_duration;
            Some(GameTime {
                now: synthetic_now,
                absolute_frame_number: (synthetic_now * 1000.0 / config().ms_per_animation_frame)
                    as u64,
                time_since_last_frame: self.frame_duration,
                excess_time_offset: 0.,
            })
        } else {
            None
        }
    }
}
