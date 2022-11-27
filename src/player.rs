use std::rc::Rc;

use macroquad::prelude::{Rect, Vec2};

use crate::{
    collision::CollisionFlags,
    config::config,
    entity::{filter_and_process_entities, Entity, EntityMap},
    game_assets::game_assets,
    hierarchy::ChildComponent,
    input::{Buttons, InputState},
    level::Level,
    life_transfer::{get_life_giving_amount_or_zero, LifeTransfer},
    materials::{replace_colors_with_image, MaterialRenderer, ReplaceColorOptions},
    physics::{PhysicsCollisionBehavior, PhysicsComponent},
    push::PushComponent,
    running::RunComponent,
    sprite_component::{LeftFacingRendering, Renderer, SpriteComponent},
    sprite_renderer::SpriteRenderer,
    time::GameTime,
    world::World,
    z_index::ZIndexComponent,
};

#[derive(Default, Copy, Clone)]
pub struct PlayerComponent {
    is_in_air: bool,
    coyote_time_start: Option<f64>,
    run_direction: f32,
    pub has_spear: bool,
    spear_point_entity: Option<u64>,
}

pub fn create_player(start_rect: Rect, name_for_debugging: &'static str) -> Entity {
    Entity {
        sprite: SpriteComponent {
            base_relative_bbox: game_assets().huntress.idle_bbox,
            ..Default::default()
        }
        .at_bottom_left(&start_rect),
        z_index: ZIndexComponent::new(500),
        player: Some(PlayerComponent {
            has_spear: true,
            ..Default::default()
        }),
        name_for_debugging: Some(name_for_debugging),
        run: Some(RunComponent::new()),
        attachment: Some(Default::default()),
        physics: PhysicsComponent {
            collision_behavior: PhysicsCollisionBehavior::Stop,
            collision_flags: CollisionFlags::ENVIRONMENT | CollisionFlags::PLAYER_ONLY,
            defies_level_bounds: true,
            ..Default::default()
        },
        push: Some(PushComponent {
            can_push: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

pub fn teleport_entity(entity: &mut Entity, pos: Vec2) {
    entity.sprite.pos = pos;
    if let Some(attachment) = entity.attachment.as_mut() {
        attachment.reset(&mut entity.physics);
    }
}

pub fn process_player_input(player: &mut Entity, time: &GameTime, input: &InputState) {
    let attachment = player.attachment.as_mut().unwrap();
    if attachment.is_attached() {
        if input.is_pressed(Buttons::JUMP) {
            attachment.detach(&mut player.physics);
        }
    } else {
        unattached_player_process_input(player, time, input);
    }
}

pub fn player_update_system(entities: &mut EntityMap, time: &GameTime) {
    let config = config();
    filter_and_process_entities(
        entities,
        |entity| entity.player.is_some(),
        |player_entity, entities, player_id| {
            let physics = &mut player_entity.physics;
            let sprite = &mut player_entity.sprite;
            let player = player_entity.player.as_mut().unwrap();
            let push = player_entity.push.as_mut().unwrap();
            let attachment = &mut player_entity.attachment.as_mut().unwrap();

            if physics.latest_frame.is_on_any_surface {
                // The player just landed (or remains on the ground).
                player.is_in_air = false;
                player.coyote_time_start = None;
                attachment.reset(physics);
            } else if !player.is_in_air {
                if let Some(coyote_start_time) = &player.coyote_time_start {
                    if time.now - coyote_start_time > config.coyote_time_ms / 1000. {
                        // The player fell off a ledge, and is out of coyote time.
                        player.is_in_air = true;
                        player.coyote_time_start = None;
                    }
                } else {
                    // Aside from the usual benefits of coyote time, this also
                    // de-jitters weird situations where the player is on a
                    // moving platform that has technically moved underneath them
                    // for a single frame.
                    player.coyote_time_start = Some(time.now);
                }
            }

            if !player.is_in_air && player.run_direction != 0. {
                sprite.is_facing_left = player.run_direction < 0.;
            }

            attachment.should_attach = player.is_in_air;
            push.can_push = !player.is_in_air;
            sprite.sprite = Some(sprite_renderer(
                player.is_in_air,
                &physics.velocity,
                player.run_direction,
            ));
            sprite.left_facing_x_offset = if attachment.is_attached() {
                // The player juts out awkwardly from their carrier if offset,
                // so don't offset.
                0.
            } else {
                config.player_left_facing_x_offset
            };
            sprite.update_looping_frame_number(time);
            if player.has_spear {
                update_spear(player, player_id, sprite, entities);
            } else {
                sprite.material =
                    replace_colors_with_image(&game_assets().huntress.no_spear_color_replacements);
            }
        },
    );
}

fn update_spear(
    player: &mut PlayerComponent,
    player_id: u64,
    sprite: &mut SpriteComponent,
    entities: &mut EntityMap,
) {
    let spear_point_entity_id = match player.spear_point_entity {
        None => {
            let spear_point_id = entities.new_id();
            entities.insert(
                spear_point_id,
                create_spear_point_entity(player_id, &sprite),
            );
            player.spear_point_entity = Some(spear_point_id);
            spear_point_id
        }
        Some(id) => id,
    };
    let spear_glow_amount = if let Some(spear_point_entity) = entities.get(spear_point_entity_id) {
        get_life_giving_amount_or_zero(spear_point_entity.life_transfer)
    } else {
        println!("Warning: invalid spear point entity id!");
        0.
    };
    sprite.material = MaterialRenderer::ReplaceColors(ReplaceColorOptions {
        image: Some((
            &game_assets().huntress.spear_glow_color_replacements,
            spear_glow_amount,
        )),
        ..Default::default()
    });
}

fn create_spear_point_entity(player_id: u64, player_sprite: &SpriteComponent) -> Entity {
    let assets = &game_assets().huntress;

    Entity {
        sprite: SpriteComponent {
            base_relative_bbox: assets.spear_point_bbox,
            left_facing_rendering: LeftFacingRendering::FlipBoundingBox,
            left_facing_bbox_x_offset: player_sprite.left_facing_x_offset,
            sprite: Some(&assets.idle),
            renderer: Renderer::Invisible,
            ..Default::default()
        },
        life_transfer: Some(LifeTransfer::Giving(0.)),
        child: Some(ChildComponent { parent: player_id }),
        ..Default::default()
    }
}

fn unattached_player_process_input(
    player_entity: &mut Entity,
    time: &GameTime,
    input: &InputState,
) {
    let time_since_last_frame = time.time_since_last_frame;
    let config = config();
    let physics = &mut player_entity.physics;
    let run = player_entity.run.as_mut().unwrap();
    let player = player_entity.player.as_mut().unwrap();
    run.update(
        time_since_last_frame,
        input.is_down(Buttons::LEFT),
        input.is_down(Buttons::RIGHT),
    );

    if player.is_in_air {
        if input.is_down(Buttons::JUMP) && physics.velocity.y < 0. {
            physics.velocity.y -=
                config.long_jump_keypress_extra_force * time_since_last_frame as f32;
        }
        if run.is_running() {
            physics.velocity.x = run.run_speed();
        }
    } else {
        if input.is_pressed(Buttons::JUMP) {
            let new_velocity = Vec2::new(run.run_speed(), -config.jump_velocity);
            physics.velocity.x = new_velocity.x;
            physics.velocity.y = new_velocity.y;
            player.is_in_air = true
        } else {
            physics.x_impulse = run.run_speed();
        }
    }
    player.run_direction = physics.x_impulse;
}

fn sprite_renderer(
    is_in_air: bool,
    velocity: &Vec2,
    run_direction: f32,
) -> &'static SpriteRenderer {
    let sprites = game_assets();
    if is_in_air {
        if velocity.y >= 0. {
            &sprites.huntress.fall
        } else {
            &sprites.huntress.jump
        }
    } else {
        if run_direction != 0. {
            &sprites.huntress.run
        } else {
            &sprites.huntress.idle
        }
    }
}

pub fn did_fall_off_level(sprite: &SpriteComponent, level: &Level) -> bool {
    sprite.bbox().top() - level.pixel_bounds().bottom() > config().fall_off_level_threshold
}

pub fn should_switch_levels(
    sprite: &SpriteComponent,
    level: &Level,
    world: &World,
) -> Option<(Rc<Level>, Vec2)> {
    if !level.contains_majority_of(&sprite.bbox()) {
        let world_pos = level.to_world_coords(&sprite.pos);
        let result = world.find_level_containing_majority_of(&world_pos, &sprite.relative_bbox());
        if result.is_some() {
            return result;
        }
    }
    None
}
