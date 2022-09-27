use macroquad::prelude::{Rect, Vec2};

use crate::{
    collision::{
        collision_resolution_loop, maybe_reverse_direction_x, maybe_reverse_direction_xy,
        process_collision, Collider, Side,
    },
    config::config,
    entity::{Entity, EntityMap},
    level::Level,
    time::GameTime,
};

#[derive(Default, PartialEq)]
pub enum PhysicsCollisionBehavior {
    #[default]
    None,
    Stop,
    ReverseDirectionX,
    ReverseDirectionXY,
}

#[derive(Default)]
pub struct PhysicsComponent {
    pub velocity: Vec2,

    /// Extra amount of x-velocity for the next iteration of the physics loop.
    /// Reset to 0 at the end of the iteration.
    pub x_impulse: f32,

    pub defies_gravity: bool,
    pub defies_level_bounds: bool,
    pub collision_behavior: PhysicsCollisionBehavior,

    /// Results of the latest iteration of the physics loop.
    /// This should really be read-only, but it's easiest to just make it public.
    pub latest_frame: PhysicsFrameResults,
}

#[derive(Default)]
pub struct PhysicsFrameResults {
    pub is_on_any_surface: bool,
    pub was_displaced: bool,
}

fn physics_collision_resolution(
    entity: &mut Entity,
    prev_bbox: &Rect,
    level: &Level,
    dynamic_colliders: &Vec<Collider>,
) -> PhysicsFrameResults {
    let physics = &mut entity.physics;
    let sprite = &mut entity.sprite;
    let mut results: PhysicsFrameResults = Default::default();

    collision_resolution_loop(|| {
        let bbox = sprite.bbox();

        let colliders = level
            .iter_colliders_ex(&bbox, !physics.defies_level_bounds)
            .chain(dynamic_colliders.iter().copied());

        for collider in colliders {
            if let Some(collision) = process_collision(&collider, &prev_bbox, &bbox) {
                match collision.side {
                    Side::Top => {
                        results.is_on_any_surface = true;
                        if !physics.defies_gravity {
                            physics.velocity.y = collider.velocity.y;
                        }
                        if physics.collision_behavior == PhysicsCollisionBehavior::Stop {
                            physics.velocity.x = collider.velocity.x;
                        }
                    }
                    Side::Bottom => {
                        if physics.collision_behavior == PhysicsCollisionBehavior::Stop {
                            physics.velocity.y = 0.;
                        }
                    }
                    Side::Left | Side::Right => {
                        if physics.collision_behavior == PhysicsCollisionBehavior::Stop {
                            physics.velocity.x = 0.;
                        }
                    }
                }

                if collision.displacement != Vec2::ZERO {
                    sprite.pos += collision.displacement;
                    results.was_displaced = true;
                    match physics.collision_behavior {
                        PhysicsCollisionBehavior::ReverseDirectionX => {
                            maybe_reverse_direction_x(
                                &mut physics.velocity,
                                &collision.displacement,
                            );
                        }
                        PhysicsCollisionBehavior::ReverseDirectionXY => {
                            maybe_reverse_direction_xy(
                                &mut physics.velocity,
                                &collision.displacement,
                            );
                        }
                        _ => {}
                    }
                    return true;
                }
            }
        }
        false
    });

    results
}

pub fn physics_system(
    entities: &mut EntityMap,
    level: &Level,
    time: &GameTime,
    dynamic_colliders: &Vec<Collider>,
) {
    let gravity = config().gravity;
    let time_since_last_frame = time.time_since_last_frame as f32;
    let gravity_this_frame = gravity * time_since_last_frame;

    for entity in entities.values_mut() {
        if !entity.physics.defies_gravity {
            entity.physics.velocity.y += gravity_this_frame;
        }

        let prev_bbox = entity.sprite.bbox();

        entity.sprite.pos += entity.physics.velocity * time_since_last_frame;
        entity.sprite.pos.x += entity.physics.x_impulse * time_since_last_frame;
        entity.physics.x_impulse = 0.;

        let results = if entity.physics.collision_behavior != PhysicsCollisionBehavior::None {
            physics_collision_resolution(entity, &prev_bbox, &level, dynamic_colliders)
        } else {
            Default::default()
        };

        entity.physics.latest_frame = results;
    }
}
