use bitflags::bitflags;
use macroquad::prelude::{is_key_down, KeyCode};

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct Buttons: u32 {
        const LEFT = 0b00000001;
        const RIGHT = 0b00000010;
        const JUMP = 0b00000100;
    }
}

impl Default for Buttons {
    fn default() -> Self {
        Buttons::empty()
    }
}

fn key_to_button(key_code: KeyCode, button: Buttons) -> Buttons {
    if is_key_down(key_code) {
        button
    } else {
        Buttons::empty()
    }
}

impl Buttons {
    pub fn from_macroquad() -> Self {
        key_to_button(KeyCode::A, Buttons::LEFT)
            | key_to_button(KeyCode::D, Buttons::RIGHT)
            | key_to_button(KeyCode::Space, Buttons::JUMP)
    }

    pub fn is_down(&self, button: Buttons) -> bool {
        !(*self & button).is_empty()
    }
}

#[derive(Default, Copy, Clone)]
pub struct InputState {
    current: Buttons,
    previous: Buttons,
}

impl InputState {
    pub fn update(&mut self, new_buttons: Buttons) {
        self.previous = self.current;
        self.current = new_buttons;
    }

    pub fn is_down(&self, button: Buttons) -> bool {
        self.current.is_down(button)
    }

    pub fn is_pressed(&self, button: Buttons) -> bool {
        self.current.is_down(button) && !self.previous.is_down(button)
    }
}

pub type InputStream = Box<dyn Iterator<Item = Buttons>>;

struct MacroquadInputStream;

impl Iterator for MacroquadInputStream {
    type Item = Buttons;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Buttons::from_macroquad())
    }
}

pub fn create_macroquad_input_stream() -> InputStream {
    Box::new(MacroquadInputStream)
}
