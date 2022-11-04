use std::iter::Fuse;

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
const DEFAULT_MAX_TIME_BETWEEN_FRAMES: f64 = 1. / 30.;

#[derive(Debug, PartialEq)]
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

#[derive(Clone, Copy)]
pub struct FixedGameTime {
    now: f64,
    frames_so_far: u64,
    start: f64,
    frame_duration: f64,
    excess_time_offset: f64,
    max_time_between_frames: f64,
    ms_per_animation_frame: f64,
    time_when_paused: Option<f64>,
}

impl FixedGameTime {
    pub fn new(fixed_frame_rate: u64, now: f64) -> Self {
        Self::new_ex(
            fixed_frame_rate,
            now,
            DEFAULT_MAX_TIME_BETWEEN_FRAMES,
            config().ms_per_animation_frame,
        )
    }

    fn new_ex(
        fixed_frame_rate: u64,
        now: f64,
        max_time_between_frames: f64,
        ms_per_animation_frame: f64,
    ) -> Self {
        FixedGameTime {
            now,
            start: now,
            frames_so_far: 0,
            frame_duration: 1. / (fixed_frame_rate as f64),
            excess_time_offset: 0.,
            max_time_between_frames,
            ms_per_animation_frame,
            time_when_paused: None,
        }
    }

    pub fn create_paused_clone(&self) -> Self {
        let mut clone = self.clone();
        clone.pause();
        clone
    }

    pub fn pause(&mut self) {
        if !self.is_paused() {
            self.time_when_paused = Some(self.now);
        }
    }

    pub fn unpause(&mut self, now: f64) {
        if let Some(time_when_paused) = self.time_when_paused {
            self.excess_time_offset += now - time_when_paused;
            self.time_when_paused = None;
        }
    }

    pub fn set_paused(&mut self, paused: bool, now: f64) {
        if paused {
            self.pause();
        } else {
            self.unpause(now);
        }
    }

    pub fn toggle_pause(&mut self, now: f64) {
        if self.is_paused() {
            self.unpause(now);
        } else {
            self.pause();
        }
    }

    pub fn is_paused(&self) -> bool {
        self.time_when_paused.is_some()
    }

    pub fn update(&mut self, now: f64) {
        if self.is_paused() {
            return;
        }
        let last_frame_time = self.now;
        self.now = now - self.excess_time_offset;
        let time_since_last_frame = self.now - last_frame_time;
        if time_since_last_frame > self.max_time_between_frames {
            // Our "real" time as reported from macroquad is has now deviated from our
            // in-game concept of time, so adjust accordingly.
            let delta = time_since_last_frame - self.max_time_between_frames;
            self.excess_time_offset += delta;
            self.now -= delta;
        }
    }

    fn next_fixed_frame(&mut self) -> Option<GameTime> {
        let time_passed = self.now - self.start;
        let total_frames = (time_passed / self.frame_duration) as u64;
        if total_frames > self.frames_so_far {
            self.frames_so_far += 1;
            let synthetic_now = self.start + (self.frames_so_far as f64) * self.frame_duration;
            Some(GameTime {
                now: synthetic_now,
                absolute_frame_number: (synthetic_now * 1000.0 / self.ms_per_animation_frame)
                    as u64,
                time_since_last_frame: self.frame_duration,
            })
        } else {
            None
        }
    }

    pub fn iter_fixed_frames<'a>(&'a mut self) -> Fuse<FixedFrameIterator<'a>> {
        (FixedFrameIterator { time: self }).fuse()
    }
}

pub struct FixedFrameIterator<'a> {
    time: &'a mut FixedGameTime,
}

impl<'a> Iterator for FixedFrameIterator<'a> {
    type Item = GameTime;

    fn next(&mut self) -> Option<Self::Item> {
        self.time.next_fixed_frame()
    }
}

#[cfg(test)]
mod tests {
    use super::{FixedGameTime, GameTime};

    fn get_frames(fixed: &mut FixedGameTime) -> Vec<GameTime> {
        fixed.iter_fixed_frames().collect::<Vec<GameTime>>()
    }

    #[test]
    fn test_it_works() {
        let mut fixed = FixedGameTime::new_ex(1, 0., 2., 100.);
        assert_eq!(get_frames(&mut fixed), vec![]);
        fixed.update(1.0);
        assert_eq!(
            get_frames(&mut fixed),
            vec![GameTime {
                now: 1.0,
                absolute_frame_number: 10,
                time_since_last_frame: 1.0
            }]
        );
        assert_eq!(get_frames(&mut fixed), vec![]);
        fixed.update(1.5);
        assert_eq!(get_frames(&mut fixed), vec![]);
        fixed.update(2.2);
        assert_eq!(
            get_frames(&mut fixed),
            vec![GameTime {
                now: 2.0,
                absolute_frame_number: 20,
                time_since_last_frame: 1.0
            }]
        );
        fixed.update(4.1);
        assert_eq!(
            get_frames(&mut fixed),
            vec![
                GameTime {
                    now: 3.0,
                    absolute_frame_number: 30,
                    time_since_last_frame: 1.0
                },
                GameTime {
                    now: 4.0,
                    absolute_frame_number: 40,
                    time_since_last_frame: 1.0
                }
            ]
        );
        fixed.toggle_pause(4.1);
        assert_eq!(get_frames(&mut fixed), vec![]);
        fixed.toggle_pause(5.);
        assert_eq!(get_frames(&mut fixed), vec![]);
        fixed.update(5.9);
        assert_eq!(
            get_frames(&mut fixed),
            vec![GameTime {
                now: 5.0,
                absolute_frame_number: 50,
                time_since_last_frame: 1.0
            }]
        );
    }
}
