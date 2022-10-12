use macroquad::prelude::{Rect, Vec2};

use crate::{
    collision::{
        collision_resolution_loop, maybe_reverse_direction_x, maybe_reverse_direction_xy,
        process_collision, CollisionFlags, Side,
    },
    config::config,
    dynamic_collider::DynamicColliderSystem,
    entity::{Entity, EntityMap},
    level::Level,
    time::GameTime,
};

/// If we have more than this many displacements for a single entity while performing
/// collision resolution, start logging debug information.
const LOTS_OF_DISPLACEMENTS: u32 = 20;

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
    /// Whether to use the Rapier physics engine to simulate this object's physics.
    pub use_rapier: bool,

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

    /// What kinds of things the entity collides with.
    pub collision_flags: CollisionFlags,

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
    pub is_on_moving_surface: bool,
    pub was_displaced: bool,
}

pub struct PhysicsSystem {
    entities: Vec<u64>,
}

impl PhysicsSystem {
    pub fn with_capacity(capacity: usize) -> Self {
        PhysicsSystem {
            entities: Vec::with_capacity(capacity),
        }
    }

    /// Update the positions of all entities based on their velocities, applying the
    /// effects of gravity for all that obey it.
    ///
    /// After this runs, some entities may have positions that are inside others;
    /// call `physics_system_resolve_collisions` to resolve them.
    pub fn update_positions(&mut self, entities: &mut EntityMap, time: &GameTime) {
        let gravity = config().gravity;
        let time_since_last_frame = time.time_since_last_frame as f32;
        let gravity_this_frame = gravity * time_since_last_frame;

        for entity in entities.values_mut() {
            if entity.physics.use_rapier {
                continue;
            }

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
    pub fn resolve_collisions(
        &mut self,
        entities: &mut EntityMap,
        level: &Level,
        dynamic_collider_system: &mut DynamicColliderSystem,
    ) {
        let vertical_collision_leeway = config().vertical_collision_leeway;

        self.entities.clear();
        self.entities.extend(entities.keys());

        // Sort our entities from bottom to top. This ensures that any displacements
        // caused by the effects of gravity will propagate upwards, e.g. that the
        // displacement caused by a crate that hits the ground propagates to a
        // crate stacked atop it.
        self.entities.sort_by(|a, b| {
            let a_y = entities.get(a).unwrap().sprite.pos.y;
            let b_y = entities.get(b).unwrap().sprite.pos.y;
            b_y.partial_cmp(&a_y).unwrap()
        });

        for &id in self.entities.iter() {
            let entity = entities.get_mut(&id).unwrap();
            if entity.physics.use_rapier {
                continue;
            }
            let results = if entity.physics.collision_behavior != PhysicsCollisionBehavior::None {
                physics_collision_resolution(
                    id,
                    entity,
                    &level,
                    dynamic_collider_system,
                    vertical_collision_leeway,
                )
            } else {
                Default::default()
            };

            entity.physics.latest_frame = results;
        }
    }
}

fn physics_collision_resolution(
    entity_id: u64,
    entity: &mut Entity,
    level: &Level,
    dynamic_collider_system: &mut DynamicColliderSystem,
    vertical_collision_leeway: f32,
) -> PhysicsFrameResults {
    let prev_bbox = entity.physics.prev_bbox;
    let physics = &mut entity.physics;
    let sprite = &mut entity.sprite;
    let collision_flags = physics.collision_flags;
    let mut results: PhysicsFrameResults = Default::default();

    let loop_result = collision_resolution_loop(|displacements| {
        let bbox = sprite.bbox();

        let colliders = level
            .iter_colliders_ex(&bbox, !physics.defies_level_bounds)
            .chain(dynamic_collider_system.colliders().copied());

        for collider in colliders {
            if let Some(collider_entity_id) = collider.entity_id {
                if collider_entity_id == entity_id {
                    // The collider represents the collider for the entity we're
                    // processing. An entity can't collide with itself, so skip testing
                    // for a collision here.
                    continue;
                }
            }
            if (collider.flags & collision_flags).is_empty() {
                // The collider and the entity can't collide, skip this.
                continue;
            }
            if let Some(collision) =
                process_collision(&collider, &prev_bbox, &bbox, vertical_collision_leeway)
            {
                let mut hit_bottom_side = false;
                match collision.side {
                    Side::Top => {
                        results.is_on_any_surface = true;
                        if !physics.defies_gravity {
                            physics.velocity.y = collider.velocity.y;
                            if collider.velocity.y != 0. {
                                results.is_on_moving_surface = true;
                            }
                        }
                        if physics.collision_behavior == PhysicsCollisionBehavior::Stop {
                            physics.velocity.x = collider.velocity.x;
                            if collider.velocity.x != 0. {
                                results.is_on_moving_surface = true;
                            }
                        }
                    }
                    Side::Bottom => {
                        if physics.collision_behavior == PhysicsCollisionBehavior::Stop {
                            physics.velocity.y = 0.;
                        }
                        hit_bottom_side = true;
                    }
                    Side::Left | Side::Right => {
                        if physics.collision_behavior == PhysicsCollisionBehavior::Stop {
                            physics.velocity.x = 0.;
                        }
                    }
                }

                if hit_bottom_side && results.is_on_any_surface && collider.entity_id.is_some() {
                    // We are being squeezed from the top and bottom. Assume that it's
                    // gravity that's doing the squeezing; we already displaced ourself
                    // from below in a previous iteration of this loop, so return now
                    // without doing any displacement, so that whatever's above us
                    // (remember we're iterating through entities from bottom to top)
                    // will be displaced by us, if we have a dynamic collider.
                    return false;
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
                    if displacements > LOTS_OF_DISPLACEMENTS {
                        println!(
                            "WARNING: collision_resolution_loop #{} {:?} with {:?}",
                            displacements, collision, collider
                        );
                    }
                    return true;
                }
            }
        }
        false
    });

    if loop_result.aborted {
        println!(
            "WARNING: aborting collision_resolution_loop for entity {} after {} iterations.",
            entity, loop_result.displacements
        );
    }

    if results.was_displaced && entity.dynamic_collider.is_some() {
        // This entity has a dynamic collider associated with it, so update its
        // computed collider to reflect its displaced position. This will ensure
        // anything above us that collides with us is displaced by our new
        // position.
        dynamic_collider_system.update_dynamic_collider(entity_id, entity);
    }

    results
}
