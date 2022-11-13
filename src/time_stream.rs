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
