use anyhow::Result;
use macroquad::{prelude::YELLOW, text::draw_text};
use std::fmt::Write;

use crate::{config::config, fps::FpsCounter, level_runtime::LevelRuntime};

const DEBUG_TEXT_CAPACITY: usize = 3000;

pub struct DebugMode {
    text: String,
}

impl Default for DebugMode {
    fn default() -> Self {
        DebugMode {
            text: String::with_capacity(DEBUG_TEXT_CAPACITY),
        }
    }
}

impl DebugMode {
    pub fn update(
        &mut self,
        runtime: &LevelRuntime,
        fps: &FpsCounter,
        draw_fps: &FpsCounter,
    ) -> Result<()> {
        self.text.clear();

        writeln!(
            self.text,
            "fps: {}  draw fps: {}",
            fps.value(),
            draw_fps.value()
        )?;

        runtime.generate_debug_text(&mut self.text)?;

        Ok(())
    }

    pub fn draw(&self, runtime: &LevelRuntime) {
        runtime.draw_debug_layer();

        let font_size = config().debug_text_size;
        let margin = 32.;
        let x = margin;
        let mut y = margin;
        for line in self.text.split("\n") {
            draw_text(line, x, y, font_size, YELLOW);
            y += font_size;
        }
    }
}
