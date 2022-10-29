use crate::{config::config, sprite_renderer::SpriteRenderer};

/// The maximum time we'll process between drawing frames. This prevents
/// us from spending too long "catching up" with the time elapsed since
/// the last frame. It also ensures resonable behavior in situations where
/// e.g. the user puts their computer to sleep, or Macroquad's event
/// loop takes a really long time to get back to us (see e.g.
/// https://github.com/toolness/macroquad-fun/issues/4).
///
///  This does mean that really slow devices will appear to
/// run in slow motion.
///
/// (Note that if we actually want to play the game in slow motion, we can
/// set this to a ridiculously high value!)
const MAX_TIME_BETWEEN_FRAMES: f64 = 1. / 30.;

pub struct GameTime {
    pub now: f64,
    pub absolute_frame_number: u64,
    pub time_since_last_frame: f64,
}

impl GameTime {
    pub fn looping_frame_number(&self, sprite: &SpriteRenderer) -> u32 {
        (self.absolute_frame_number % sprite.num_frames() as u64) as u32
    }
}

pub struct FixedGameTime {
    now: f64,
    frames_so_far: u64,
    start: f64,
    frame_duration: f64,
    excess_time_offset: f64,
}

impl FixedGameTime {
    pub fn new(fixed_frame_rate: u64, now: f64) -> Self {
        FixedGameTime {
            now,
            start: now,
            frames_so_far: 0,
            frame_duration: 1. / (fixed_frame_rate as f64),
            excess_time_offset: 0.,
        }
    }

    pub fn update(&mut self, now: f64) {
        let last_frame_time = self.now;
        self.now = now - self.excess_time_offset;
        let time_since_last_frame = self.now - last_frame_time;
        if time_since_last_frame > MAX_TIME_BETWEEN_FRAMES {
            // Our "real" time as reported from macroquad is has now deviated from our
            // in-game concept of time, so adjust accordingly.
            let delta = time_since_last_frame - MAX_TIME_BETWEEN_FRAMES;
            self.excess_time_offset += delta;
            self.now -= delta;
        }
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
            })
        } else {
            None
        }
    }
}
