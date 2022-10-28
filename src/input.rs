use macroquad::prelude::{is_key_down, KeyCode};

#[derive(Default, Copy, Clone)]
pub struct InputState {
    pub is_left_key_down: bool,
    pub is_right_key_down: bool,
    pub is_jump_key_down: bool,
    pub is_jump_key_pressed: bool,
}

impl InputState {
    pub fn from_macroquad(prev_state: InputState) -> Self {
        let is_jump_key_down = is_key_down(KeyCode::Space);
        InputState {
            is_left_key_down: is_key_down(KeyCode::A),
            is_right_key_down: is_key_down(KeyCode::D),
            is_jump_key_down,
            is_jump_key_pressed: is_jump_key_down && !prev_state.is_jump_key_down,
        }
    }
}
