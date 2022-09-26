use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode, Rect, Vec2};

use crate::{
    config::config,
    entity::{Entity, EntityMap, EntityMapHelpers},
    game_sprites::game_sprites,
    level::Level,
    physics::{PhysicsCollisionBehavior, PhysicsComponent},
    running::RunComponent,
    sprite_component::{Renderer, SpriteComponent},
    sprite_renderer::SpriteRenderer,
    time::GameTime,
    world::world,
    z_index::ZIndexComponent,
};

#[derive(Default)]
pub struct PlayerComponent {
    is_in_air: bool,
    coyote_time_start: Option<f64>,
    run_direction: f32,
}

pub fn create_player(start_rect: Rect) -> Entity {
    Entity {
        sprite: SpriteComponent {
            relative_bbox: game_sprites().huntress.idle_bbox,
            ..Default::default()
        }
        .at_bottom_left(&start_rect),
        z_index: ZIndexComponent::new(500),
        player: Some(PlayerComponent {
            ..Default::default()
        }),
        run: Some(RunComponent::new()),
        attachment: Some(Default::default()),
        physics: PhysicsComponent {
            collision_behavior: PhysicsCollisionBehavior::Stop,
            defies_level_bounds: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn teleport_entity(entity: &mut Entity, pos: Vec2) {
    entity.sprite.pos = pos;
    if let Some(attachment) = entity.attachment.as_mut() {
        attachment.reset();
    }
}

pub fn process_player_input(entities: &mut EntityMap, time: &GameTime) {
    let player = entities.player_mut();
    let attachment = player.attachment.as_mut().unwrap();
    if attachment.is_attached() {
        if is_key_pressed(KeyCode::Space) {
            attachment.detach(&mut player.physics);
        }
    } else {
        unattached_player_process_input(player, time);
    }
}

pub fn player_update_system(entities: &mut EntityMap, time: &GameTime) {
    let player_entity = entities.player_mut();
    let physics = &mut player_entity.physics;
    let sprite = &mut player_entity.sprite;
    let player = player_entity.player.as_mut().unwrap();
    let attachment = &mut player_entity.attachment.as_mut().unwrap();

    if physics.latest_frame.is_on_any_surface {
        // The player just landed (or remains on the ground).
        player.is_in_air = false;
        player.coyote_time_start = None;
        attachment.reset();
    } else if !player.is_in_air {
        if let Some(coyote_start_time) = &player.coyote_time_start {
            if time.now - coyote_start_time > config().coyote_time_ms / 1000. {
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
    sprite.renderer = Renderer::Sprite(sprite_renderer(
        player.is_in_air,
        &physics.velocity,
        player.run_direction,
    ));
    sprite.update_looping_frame_number(time);
}

fn unattached_player_process_input(player_entity: &mut Entity, time: &GameTime) {
    let time_since_last_frame = time.time_since_last_frame;
    let config = config();
    let physics = &mut player_entity.physics;
    let run = player_entity.run.as_mut().unwrap();
    let player = player_entity.player.as_mut().unwrap();
    run.update(
        time_since_last_frame,
        is_key_down(KeyCode::A),
        is_key_down(KeyCode::D),
    );

    if player.is_in_air {
        if is_key_down(KeyCode::Space) && physics.velocity.y < 0. {
            physics.velocity.y -=
                config.long_jump_keypress_extra_force * time_since_last_frame as f32;
        }
        if run.is_running() {
            physics.velocity.x = run.run_speed();
        }
    } else {
        if is_key_pressed(KeyCode::Space) {
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
    let sprites = game_sprites();
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
) -> Option<(&'static Level, Vec2)> {
    let world = world();
    if !level.contains_majority_of(&sprite.bbox()) {
        let world_pos = level.to_world_coords(&sprite.pos);
        let result = world.find_level_containing_majority_of(&world_pos, &sprite.relative_bbox);
        if result.is_some() {
            return result;
        }
    }
    None
}
