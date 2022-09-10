use crate::{flying_eye::FlyingEye, level::Level};

pub struct LevelRuntime {
    pub level: &'static Level,
    pub flying_eyes: Vec<FlyingEye>,
}

impl LevelRuntime {
    pub fn new(level: &'static Level) -> Self {
        LevelRuntime {
            level,
            flying_eyes: level.spawn_flying_eyes(),
        }
    }
}
