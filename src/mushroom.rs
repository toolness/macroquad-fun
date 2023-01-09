use macroquad::prelude::Rect;

use crate::{
    animator::Animator,
    audio::play_sound_effect,
    config::config,
    dynamic_collider::{DynamicColliderComponent, RelativeCollider},
    entity::{filter_and_process_entities, Entity, EntityMap},
    game_assets::game_assets,
    life_transfer::{get_life_receiving_amount_or_zero, LifeTransfer},
    materials::{replace_colors_with_image, LerpType, MaterialRenderer, ReplaceColorOptions},
    physics::{PhysicsCollisionBehavior, PhysicsComponent},
    sprite_component::{LeftFacingRendering, SpriteComponent},
    time::GameTime,
};

#[derive(Clone, Copy)]
pub struct MushroomComponent {
    state: MushroomState,
}

#[derive(Clone, Copy)]
pub enum MushroomState {
    Dead,
    Rezzing(Animator),
    Alive,
}

fn dead_frame() -> u32 {
    game_assets().mushroom.death.last_frame()
}

pub fn create_mushrom(start_rect: Rect) -> Entity {
    let assets = &game_assets().mushroom;
    let death_sprite = &assets.death;
    Entity {
        sprite: SpriteComponent {
            base_relative_bbox: assets.dead_bbox,
            sprite: Some(&death_sprite),
            left_facing_rendering: LeftFacingRendering::FlipBoundingBox,
            ..Default::default()
        }
        .at_bottom_left(&start_rect),
        mushroom: Some(MushroomComponent {
            state: MushroomState::Dead,
        }),
        physics: PhysicsComponent {
            collision_behavior: PhysicsCollisionBehavior::ReverseDirectionX,
            gravity_coefficient: Some(0.5),
            ..Default::default()
        },
        life_transfer: Some(LifeTransfer::Receiving(0.)),
        ..Default::default()
    }
}

pub fn mushroom_movement_system(entities: &mut EntityMap, time: &GameTime) {
    filter_and_process_entities(
        entities,
        |entity| entity.mushroom.is_some(),
        |entity, _entities, _id| {
            update_mushroom(entity, time);
        },
    );
}

fn update_mushroom(entity: &mut Entity, time: &GameTime) {
    let mushroom = entity.mushroom.as_mut().unwrap();
    let velocity = &mut entity.physics.velocity;
    let sprite = &mut entity.sprite;
    let dynamic_collider = &mut entity.dynamic_collider;
    let config = config();
    let assets = &game_assets().mushroom;

    match &mushroom.state {
        MushroomState::Dead => {
            let life_receiving = get_life_receiving_amount_or_zero(entity.life_transfer);
            if life_receiving == 1.0 {
                mushroom.state = MushroomState::Rezzing(
                    Animator::new(dead_frame(), true, &time)
                        .with_ms_per_animation_frame(config.mushroom_rez_ms_per_animation_frame),
                );
                entity.life_transfer = None;
                sprite.base_relative_bbox = assets.idle_bbox;
                play_sound_effect(game_assets().mushroom.rez_sound);
            }
        }
        MushroomState::Rezzing(animator) => {
            if animator.is_done(&time) {
                mushroom.state = MushroomState::Alive;
                sprite.sprite = Some(&assets.run);
                sprite.material = replace_colors_with_image(&assets.color_replacements);
                velocity.x = config.mushroom_speed;
                let _ = dynamic_collider.insert(DynamicColliderComponent::new(RelativeCollider {
                    rect: assets.platform_bbox,
                    enable_top: true,
                    ..Default::default()
                }));
            }
        }
        MushroomState::Alive => {
            sprite.is_facing_left = velocity.x < 0.;
        }
    }
    mushroom.set_sprite(time, sprite, entity.life_transfer);
}

impl MushroomComponent {
    fn set_sprite(
        &self,
        time: &GameTime,
        sprite: &mut SpriteComponent,
        life_transfer: Option<LifeTransfer>,
    ) {
        match &self.state {
            MushroomState::Dead => {
                sprite.current_frame_number = dead_frame();
                let glow_image = &game_assets().huntress.spear_glow_color_replacements;
                let glow_color = glow_image.get_pixel((glow_image.width as u32) - 1, 0);
                let amount = get_life_receiving_amount_or_zero(life_transfer);
                sprite.material = MaterialRenderer::ReplaceColors(ReplaceColorOptions {
                    image: Some((&game_assets().mushroom.dead_color_replacements, 1.)),
                    lerp: Some((LerpType::ReplacedColor, glow_color, amount)),
                })
            }
            MushroomState::Rezzing(animator) => {
                let curr_frame = animator.get_frame(&time);
                let amount = curr_frame as f32 / animator.last_frame() as f32;
                sprite.current_frame_number = curr_frame;
                if let MaterialRenderer::ReplaceColors(options) = &mut sprite.material {
                    if let Some(lerp_options) = &mut options.lerp {
                        lerp_options.2 = amount;
                    }
                }
            }
            MushroomState::Alive => {
                sprite.update_looping_frame_number(&time);
            }
        }
    }
}
