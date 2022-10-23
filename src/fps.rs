#[derive(Default)]
/// Macroquad's get_fps() fluctuates ridiculously which makes it difficult
/// to read, so we'll roll our own.
pub struct FpsCounter {
    last_second_start_time: f64,
    frames_this_second: i32,
    fps: i32,
}

impl FpsCounter {
    pub fn update(&mut self, now: f64) {
        let start_delta = now - self.last_second_start_time;
        if start_delta > 1. {
            self.fps = self.frames_this_second;
            self.frames_this_second = 1;
            self.last_second_start_time = now;
        } else {
            self.frames_this_second += 1;
        }
    }

    pub fn value(&self) -> i32 {
        self.fps
    }
}
