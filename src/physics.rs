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
    /// This effectively disables collisions for the entity.
    None,

    /// When the entity collides with a Collider, make it stop.
    Stop,

    /// When the entity collides with a Collider, reverse its direction along the X axis.
    ReverseDirectionX,

    /// When the entity collides with a Collider, reverse its direction along the X and Y axes.
    ReverseDirectionXY,
}

#[derive(Default)]
pub struct PhysicsComponent {
    /// The current velocity of the entity.
    pub velocity: Vec2,

    /// Extra amount of x-velocity for the next iteration of the physics loop.
    /// Reset to 0 at the end of the iteration.
    pub x_impulse: f32,

    /// How much gravity affects the y-velocity of the entity (assuming `defies_gravity` is false).
    /// It defaults to 1.0; a higher value amplifies gravity, while a lower value dampens it.
    ///
    /// This ideally should be a f32 instead of an Option<f32>, but I'm too lazy to fiddle with
    /// the Default trait to make this default to 1.0.
    pub gravity_coefficient: Option<f32>,

    /// If false, the entity will be subject to the forces of gravity.
    pub defies_gravity: bool,

    /// If false, the bounds of the level itself will be treated as colliders for this entity.
    pub defies_level_bounds: bool,

    /// What happens when the entity collides with a Collider.
    pub collision_behavior: PhysicsCollisionBehavior,

    /// The bounding box of the entity in the *last* frame.
    /// This should really be read-only, but it's easiest to just make it public.
    pub prev_bbox: Rect,

    /// Results of the latest iteration of the physics loop.
    /// This should really be read-only, but it's easiest to just make it public.
    pub latest_frame: PhysicsFrameResults,
}

#[derive(Default)]
pub struct PhysicsFrameResults {
    pub is_on_any_surface: bool,
    pub was_displaced: bool,
}

/// Update the positions of all entities based on their velocities, applying the
/// effects of gravity for all that obey it.
///
/// After this runs, some entities may have positions that are inside others;
/// call `physics_system_resolve_collisions` to resolve them.
pub fn physics_system_update_positions(entities: &mut EntityMap, time: &GameTime) {
    let gravity = config().gravity;
    let time_since_last_frame = time.time_since_last_frame as f32;
    let gravity_this_frame = gravity * time_since_last_frame;

    for entity in entities.values_mut() {
        if !entity.physics.defies_gravity {
            entity.physics.velocity.y +=
                gravity_this_frame * entity.physics.gravity_coefficient.unwrap_or(1.0);
        }

        entity.physics.prev_bbox = entity.sprite.bbox();

        entity.sprite.pos += entity.physics.velocity * time_since_last_frame;
        entity.sprite.pos.x += entity.physics.x_impulse * time_since_last_frame;
        entity.physics.x_impulse = 0.;
    }
}

/// Resolve any collisions that occurred since the last call to
/// `physics_system_update_positions`.
pub fn physics_system_resolve_collisions(
    entities: &mut EntityMap,
    level: &Level,
    dynamic_colliders: &Vec<Collider>,
) {
    for (&id, entity) in entities.iter_mut() {
        let results = if entity.physics.collision_behavior != PhysicsCollisionBehavior::None {
            physics_collision_resolution(id, entity, &level, dynamic_colliders)
        } else {
            Default::default()
        };

        entity.physics.latest_frame = results;
    }
}

fn physics_collision_resolution(
    entity_id: u64,
    entity: &mut Entity,
    level: &Level,
    dynamic_colliders: &Vec<Collider>,
) -> PhysicsFrameResults {
    let prev_bbox = entity.physics.prev_bbox;
    let physics = &mut entity.physics;
    let sprite = &mut entity.sprite;
    let mut results: PhysicsFrameResults = Default::default();

    collision_resolution_loop(|| {
        let bbox = sprite.bbox();

        let colliders = level
            .iter_colliders_ex(&bbox, !physics.defies_level_bounds)
            .chain(dynamic_colliders.iter().copied());

        for collider in colliders {
            if let Some(collider_entity_id) = collider.entity_id {
                if collider_entity_id == entity_id {
                    // The collider represents the collider for the entity we're
                    // processing. An entity can't collide with itself, so skip testing
                    // for a collision here.
                    continue;
                }
            }
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
