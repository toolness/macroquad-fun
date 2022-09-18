use std::collections::HashMap;

use macroquad::prelude::Vec2;

use crate::{
    attachment::{AttachableComponent, AttachmentComponent},
    flying_eye::FlyingEyeComponent,
    mushroom::MushroomComponent,
    player::PlayerComponent,
    running::RunComponent,
    sprite_component::SpriteComponent,
};

#[derive(Default)]
pub struct Entity {
    pub sprite: SpriteComponent,
    pub velocity: Vec2,
    pub mushroom: Option<MushroomComponent>,
    pub flying_eye: Option<FlyingEyeComponent>,
    pub attachable: Option<AttachableComponent>,
    pub player: Option<PlayerComponent>,
    pub run: Option<RunComponent>,
    pub attachment: Option<AttachmentComponent>,
}

pub type EntityMap = HashMap<u64, Entity>;
