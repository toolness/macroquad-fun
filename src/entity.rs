use macroquad::prelude::Vec2;

use crate::{
    attachment::AttachableComponent, flying_eye::FlyingEyeComponent, mushroom::MushroomComponent,
    sprite_component::SpriteComponent,
};

#[derive(Default)]
pub struct Entity {
    pub sprite: SpriteComponent,
    pub velocity: Vec2,
    pub mushroom: Option<MushroomComponent>,
    pub flying_eye: Option<FlyingEyeComponent>,
    pub attachable: Option<AttachableComponent>,
}
