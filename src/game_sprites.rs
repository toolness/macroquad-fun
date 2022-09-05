use anyhow::Result;
use macroquad::texture::load_texture;

use crate::sprite::Sprite;

pub struct HuntressSprites {
    pub idle: Sprite,
    pub run: Sprite,
    pub jump: Sprite,
    pub fall: Sprite,
}

pub struct GameSprites {
    pub huntress: HuntressSprites,
}

pub async fn load_game_sprites(scale: f32) -> Result<GameSprites> {
    let sprites = GameSprites {
        huntress: HuntressSprites {
            idle: Sprite::new(load_texture("media/Huntress/Idle.png").await?, 8, scale),
            run: Sprite::new(load_texture("media/Huntress/Run.png").await?, 8, scale),
            jump: Sprite::new(load_texture("media/Huntress/Jump.png").await?, 2, scale),
            fall: Sprite::new(load_texture("media/Huntress/Fall.png").await?, 2, scale),
        },
    };

    Ok(sprites)
}
