use crate::{flying_eye::FlyingEye, level::Level};

pub struct LevelRuntime<'a> {
    pub level: &'a Level,
    pub flying_eyes: Vec<FlyingEye>,
}

impl<'a> LevelRuntime<'a> {
    pub fn new(level: &'a Level) -> Self {
        LevelRuntime {
            level,
            flying_eyes: level.spawn_flying_eyes(),
        }
    }
}
