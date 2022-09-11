use std::collections::HashMap;

use crate::{flying_eye::FlyingEye, level::Level};

pub struct LevelRuntime {
    pub level: &'static Level,
    pub flying_eyes: HashMap<u64, FlyingEye>,
    next_id: u64,
}

impl LevelRuntime {
    pub fn new(level: &'static Level) -> Self {
        let mut instance = LevelRuntime {
            level,
            flying_eyes: HashMap::new(),
            next_id: 1,
        };
        instance.change_level(&level);
        instance
    }

    pub fn change_level(&mut self, level: &'static Level) {
        self.level = level;
        self.flying_eyes.clear();
        level.spawn_entities(self);
    }

    pub fn new_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}
