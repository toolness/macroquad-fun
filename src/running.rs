use macroquad::prelude::clamp;

use crate::config::config;

pub struct RunComponent {
    run_duration: f64,
    x_direction: f32,
    prev_x_direction: f32,
    run_speed: f32,
}

impl RunComponent {
    pub fn new() -> Self {
        RunComponent {
            run_duration: 0.,
            x_direction: 0.,
            prev_x_direction: 0.,
            run_speed: 0.,
        }
    }

    pub fn update(
        &mut self,
        time_since_last_frame: f64,
        is_pressing_left: bool,
        is_pressing_right: bool,
    ) {
        let config = config();
        self.x_direction = if is_pressing_left {
            -1.
        } else if is_pressing_right {
            1.
        } else {
            0.
        } as f32;
        if self.x_direction == self.prev_x_direction {
            self.run_duration += time_since_last_frame;
        } else {
            self.run_duration = time_since_last_frame;
        }
        self.prev_x_direction = self.x_direction;
        self.run_speed = (clamp(self.run_duration * 1000.0, 0., config.ms_to_max_run_speed)
            / config.ms_to_max_run_speed)
            .powi(2) as f32
            * config.run_speed;
    }

    pub fn run_speed(&self) -> f32 {
        self.run_speed * self.x_direction
    }

    pub fn is_running(&self) -> bool {
        self.x_direction != 0.
    }
}
