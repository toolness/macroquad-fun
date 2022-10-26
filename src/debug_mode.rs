use anyhow::Result;
use macroquad::{prelude::YELLOW, text::draw_text};
use std::fmt::Write;

use crate::{config::config, fps::FpsCounter, level_runtime::LevelRuntime};

const DEBUG_TEXT_CAPACITY: usize = 3000;

pub struct DebugMode {
    debug_text_lines: Option<String>,
    fps: FpsCounter,
}

impl Default for DebugMode {
    fn default() -> Self {
        DebugMode {
            debug_text_lines: None,
            fps: FpsCounter::default(),
        }
    }
}

impl DebugMode {
    pub fn update(&mut self, runtime: &LevelRuntime, now: f64) -> Result<()> {
        self.fps.update(now);

        let text = self
            .debug_text_lines
            .get_or_insert_with(|| String::with_capacity(DEBUG_TEXT_CAPACITY));
        text.clear();

        writeln!(text, "fps: {}", self.fps.value())?;

        runtime.generate_debug_text(text)?;

        Ok(())
    }

    pub fn draw(&self, runtime: &LevelRuntime) {
        runtime.draw_debug_layer();

        if let Some(text) = &self.debug_text_lines {
            let font_size = config().debug_text_size;
            let margin = 32.;
            let x = margin;
            let mut y = margin;
            for line in text.split("\n") {
                draw_text(line, x, y, font_size, YELLOW);
                y += font_size;
            }
        }
    }
}
