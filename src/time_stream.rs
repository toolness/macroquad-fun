use macroquad::time::get_time;

pub type TimeStream = Box<dyn Iterator<Item = f64>>;

pub struct RealTimeStream;

impl Iterator for RealTimeStream {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        Some(get_time())
    }
}

pub fn create_real_time_stream() -> TimeStream {
    Box::new(RealTimeStream)
}

pub struct FixedFpsTimeStream {
    next_time: f64,
    fps: u64,
}

impl Iterator for FixedFpsTimeStream {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let time = self.next_time;
        self.next_time += 1.0 / self.fps as f64;
        Some(time)
    }
}

pub fn create_fixed_fps_time_stream(fps: u64) -> TimeStream {
    Box::new(FixedFpsTimeStream { next_time: 0., fps })
}
