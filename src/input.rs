use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode};

pub struct InputState {
    pub is_left_key_down: bool,
    pub is_right_key_down: bool,
    pub is_jump_key_down: bool,
    pub is_jump_key_pressed: bool,
}

impl InputState {
    pub fn from_macroquad() -> Self {
        InputState {
            is_left_key_down: is_key_down(KeyCode::A),
            is_right_key_down: is_key_down(KeyCode::D),
            is_jump_key_down: is_key_down(KeyCode::Space),
            is_jump_key_pressed: is_key_pressed(KeyCode::Space),
        }
    }
}
