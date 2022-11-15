use macroquad::prelude::{clamp, Rect, Vec2};

use crate::{
    animator::Animator,
    config::config,
    dynamic_collider::{DynamicColliderComponent, RelativeCollider},
    entity::{filter_and_process_entities, Entity, EntityMap},
    game_assets::game_assets,
    materials::{replace_colors_with_image, LerpType, MaterialRenderer, ReplaceColorOptions},
    physics::{PhysicsCollisionBehavior, PhysicsComponent},
    sprite_component::{LeftFacingRendering, Renderer, SpriteComponent},
    time::GameTime,
};

#[derive(Clone, Copy)]
pub struct MushroomComponent {
    state: MushroomState,
}

#[derive(Clone, Copy)]
pub enum MushroomState {
    Dead(f32),
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
            base_relative_bbox: assets.idle_bbox,
            renderer: Renderer::Sprite(&death_sprite),
            material: replace_colors_with_image(&assets.color_replacements),
            left_facing_rendering: LeftFacingRendering::FlipBoundingBox,
            ..Default::default()
        }
        .at_bottom_left(&start_rect),
        mushroom: Some(MushroomComponent {
            state: MushroomState::Dead(0.0),
        }),
        physics: PhysicsComponent {
            collision_behavior: PhysicsCollisionBehavior::ReverseDirectionX,
            gravity_coefficient: Some(0.5),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn mushroom_movement_system(entities: &mut EntityMap, time: &GameTime) {
    filter_and_process_entities(
        entities,
        |entity| entity.mushroom.is_some(),
        |entity, _entities| {
            let mushroom = entity.mushroom.as_mut().unwrap();
            let velocity = &mut entity.physics.velocity;
            let sprite = &mut entity.sprite;
            let dynamic_collider = &mut entity.dynamic_collider;
            update_mushroom(mushroom, velocity, sprite, dynamic_collider, time);
        },
    );

    filter_and_process_entities(
        entities,
        |entity| entity.player.is_some(),
        |player_entity, entities| {
            let config = config();
            let player = player_entity.player.as_mut().unwrap();
            let mut max_glow_amount = 0.;
            if player.has_spear {
                let player_center = player_entity.sprite.bbox().center();
                for (_id, entity) in entities.iter_mut() {
                    let Some(mushroom) = entity.mushroom.as_mut() else {
                        continue;
                    };
                    if let MushroomState::Dead(_) = mushroom.state {
                        let distance = clamp(
                            entity.sprite.bbox().center().distance(player_center)
                                - config.spear_glow_min_radius,
                            0.001,
                            config.spear_glow_max_radius,
                        );
                        let oscillator = (1.
                            + (time.now as f32 * config.spear_glow_speed_coefficient).sin())
                            / 2.;
                        let glow_amount =
                            oscillator * (1. - distance / config.spear_glow_max_radius);
                        if glow_amount > max_glow_amount {
                            max_glow_amount = glow_amount;
                        }
                        if glow_amount >= config.spear_glow_revive_threshold {
                            mushroom.state = MushroomState::Rezzing(
                                Animator::new(dead_frame(), true, &time)
                                    .with_ms_per_animation_frame(
                                        config.mushroom_rez_ms_per_animation_frame,
                                    ),
                            );
                        } else {
                            mushroom.state = MushroomState::Dead(glow_amount);
                        }
                    }
                }
                player.spear_glow_amount = max_glow_amount;
            }
        },
    );
}

fn update_mushroom(
    mushroom: &mut MushroomComponent,
    velocity: &mut Vec2,
    sprite: &mut SpriteComponent,
    dynamic_collider: &mut Option<DynamicColliderComponent>,
    time: &GameTime,
) {
    match &mushroom.state {
        MushroomState::Dead(_) => {}
        MushroomState::Rezzing(animator) => {
            if animator.is_done(&time) {
                mushroom.state = MushroomState::Alive;
                sprite.renderer = Renderer::Sprite(&game_assets().mushroom.run);
                sprite.material = MaterialRenderer::None;
                velocity.x = config().mushroom_speed;
                let _ = dynamic_collider.insert(DynamicColliderComponent::new(RelativeCollider {
                    rect: game_assets().mushroom.platform_bbox,
                    enable_top: true,
                    ..Default::default()
                }));
            }
        }
        MushroomState::Alive => {
            sprite.is_facing_left = velocity.x < 0.;
        }
    }
    mushroom.set_sprite(time, sprite);
}

impl MushroomComponent {
    fn set_sprite(&self, time: &GameTime, sprite: &mut SpriteComponent) {
        match &self.state {
            MushroomState::Dead(amount) => {
                sprite.current_frame_number = dead_frame();
                let glow_image = &game_assets().huntress.spear_glow_color_replacements;
                let glow_color = glow_image.get_pixel((glow_image.width as u32) - 1, 0);
                sprite.material = MaterialRenderer::ReplaceColors(ReplaceColorOptions {
                    image: None,
                    lerp: Some((LerpType::AllColors, glow_color, *amount)),
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
