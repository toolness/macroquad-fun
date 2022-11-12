use macroquad::prelude::{Rect, Vec2};

use crate::{
    collision::{
        collision_resolution_loop, maybe_reverse_direction_x, maybe_reverse_direction_xy,
        process_collision, Collider, CollisionFlags, Side,
    },
    config::config,
    dynamic_collider::DynamicColliderSystem,
    entity::{Entity, EntityMap, HeaplessEntityVec},
    level::Level,
    time::GameTime,
};

/// If we have more than this many displacements for a single entity while performing
/// collision resolution, start logging debug information.
const LOTS_OF_DISPLACEMENTS: u32 = 20;

#[derive(Default, PartialEq, Clone, Copy)]
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

#[derive(Default, Clone, Copy)]
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

    /// What kinds of things the entity collides with.
    pub collision_flags: CollisionFlags,

    /// The bounding box of the entity in the *last* frame.
    /// This should really be read-only, but it's easiest to just make it public.
    pub prev_bbox: Rect,

    /// Results of the latest iteration of the physics loop.
    /// This should really be read-only, but it's easiest to just make it public.
    pub latest_frame: PhysicsFrameResults,
}

#[derive(Default, Clone, Copy)]
pub struct PhysicsFrameResults {
    pub is_on_any_surface: bool,
    pub is_on_moving_surface: bool,
    pub was_displaced: bool,
    pub is_penetrating_collider: bool,
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

    for (_id, entity) in entities.iter_mut() {
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
    dynamic_collider_system: &mut DynamicColliderSystem,
) {
    let vertical_collision_leeway = config().vertical_collision_leeway;

    let mut entities_to_process: HeaplessEntityVec = heapless::Vec::new();
    entities_to_process.extend(entities.ids().cloned());

    // Sort our entities from bottom to top. This ensures that any displacements
    // caused by the effects of gravity will propagate upwards, e.g. that the
    // displacement caused by a crate that hits the ground propagates to a
    // crate stacked atop it.
    entities_to_process.sort_by(|a, b| {
        let a_y = entities.get(*a).unwrap().sprite.pos.y;
        let b_y = entities.get(*b).unwrap().sprite.pos.y;
        b_y.partial_cmp(&a_y).unwrap()
    });

    for id in entities_to_process {
        let entity = entities.get_mut(id).unwrap();
        let results = if entity.physics.collision_behavior != PhysicsCollisionBehavior::None {
            let defies_level_bounds = entity.physics.defies_level_bounds;
            physics_collision_resolution(
                id,
                entity,
                |bbox| {
                    level
                        .iter_colliders_ex(bbox, !defies_level_bounds)
                        .chain(dynamic_collider_system.colliders().copied())
                },
                vertical_collision_leeway,
            )
        } else {
            Default::default()
        };

        if results.was_displaced && entity.dynamic_collider.is_some() {
            // This entity has a dynamic collider associated with it, so update its
            // computed collider to reflect its displaced position. This will ensure
            // anything above us that collides with us is displaced by our new
            // position.
            dynamic_collider_system.update_dynamic_collider(id, entity);
        }

        entity.physics.latest_frame = results;
    }
}

fn physics_collision_resolution<F: Fn(&Rect) -> I, I: Iterator<Item = Collider>>(
    entity_id: u64,
    entity: &mut Entity,
    iter_colliders: F,
    vertical_collision_leeway: f32,
) -> PhysicsFrameResults {
    let prev_bbox = entity.physics.prev_bbox;
    let physics = &mut entity.physics;
    let sprite = &mut entity.sprite;
    let collision_flags = physics.collision_flags;
    let mut results: PhysicsFrameResults = Default::default();

    let loop_result = collision_resolution_loop(|displacements| {
        let bbox = sprite.bbox();

        for collider in iter_colliders(&bbox) {
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

            // If we've had lots of displacements, it could be because we're sandwiched between two colliders.
            // Since we normally prioritize vertical collisions over horizontal ones, let's see if we can
            // prioritize horizontal displacement to see if it gets us out of an infinite displacement loop.
            if displacements > LOTS_OF_DISPLACEMENTS
                && (collider.enable_top || collider.enable_bottom)
            {
                let horizontal_collider = Collider {
                    enable_top: false,
                    enable_bottom: false,
                    ..collider
                };
                if let Some(collision) =
                    process_collision(&horizontal_collider, &prev_bbox, &bbox, 0.)
                {
                    sprite.pos += collision.displacement;
                }
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
        results.is_penetrating_collider = true;
    }

    results
}

#[cfg(test)]
mod tests {
    use macroquad::prelude::{Rect, Vec2};

    use crate::{
        collision::{Collider, CollisionFlags, EXTRA_DISPLACEMENT},
        entity::Entity,
        physics::PhysicsComponent,
        sprite_component::SpriteComponent,
    };

    use super::{physics_collision_resolution, PhysicsFrameResults};

    const SIMPLE_ENTITY_ID: u64 = 1;
    const SIMPLE_DYNAMIC_COLLIDER_ENTITY_ID: u64 = 2;

    fn simple_collision_resolution(
        entity: &mut Entity,
        colliders: Vec<Collider>,
    ) -> PhysicsFrameResults {
        physics_collision_resolution(
            SIMPLE_ENTITY_ID,
            entity,
            |_bbox| colliders.iter().copied(),
            0.,
        )
    }

    fn make_simple_10x10_entity() -> Entity {
        let sprite = SpriteComponent {
            pos: Vec2::new(10., 10.),
            base_relative_bbox: Rect::new(0., 0., 10., 10.),
            ..Default::default()
        };
        Entity {
            sprite,
            physics: PhysicsComponent {
                collision_flags: CollisionFlags::ENVIRONMENT,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_previous_velocity(Vec2::ZERO)
    }

    fn make_simple_collider(rect: Rect) -> Collider {
        Collider {
            rect,
            prev_rect: rect,
            flags: CollisionFlags::ENVIRONMENT,
            enable_top: true,
            enable_bottom: true,
            enable_left: true,
            enable_right: true,
            ..Default::default()
        }
    }

    fn make_simple_dynamic_collider(rect: Rect, velocity: Vec2) -> Collider {
        Collider {
            rect,
            prev_rect: rect.offset(-velocity),
            entity_id: Some(SIMPLE_DYNAMIC_COLLIDER_ENTITY_ID),
            flags: CollisionFlags::ENVIRONMENT,
            enable_top: true,
            enable_bottom: true,
            enable_left: true,
            enable_right: true,
            ..Default::default()
        }
    }

    trait EntityHelpers
    where
        Self: Sized,
    {
        fn into_entity(self) -> Entity;

        fn with_previous_velocity(self, velocity: Vec2) -> Entity {
            let mut entity = self.into_entity();
            let prev_bbox = entity.sprite.bbox().offset(-velocity);
            entity.physics.velocity = velocity;
            entity.physics.prev_bbox = prev_bbox;
            entity
        }
    }

    impl<T: Into<Entity>> EntityHelpers for T {
        fn into_entity(self) -> Entity {
            self.into()
        }
    }

    trait BoundingBoxHelpers {
        fn get_bounding_box(&self) -> Rect;

        fn is_just_left_of<T: BoundingBoxHelpers>(&self, other: T) -> bool {
            let my_bbox = self.get_bounding_box();
            let other_bbox = other.get_bounding_box();
            let delta = other_bbox.left() - my_bbox.right();
            !my_bbox.overlaps(&other_bbox) && delta > 0. && delta < EXTRA_DISPLACEMENT * 2.
        }

        fn offset_right_by(&self, amount: u32) -> Rect {
            self.get_bounding_box().offset(Vec2::new(amount as f32, 0.))
        }

        fn offset_up_by(&self, amount: u32) -> Rect {
            self.get_bounding_box()
                .offset(Vec2::new(0., -(amount as f32)))
        }

        fn offset_down_by(&self, amount: u32) -> Rect {
            self.get_bounding_box().offset(Vec2::new(0., amount as f32))
        }
    }

    impl BoundingBoxHelpers for Rect {
        fn get_bounding_box(&self) -> Rect {
            *self
        }
    }

    impl BoundingBoxHelpers for Entity {
        fn get_bounding_box(&self) -> Rect {
            self.sprite.bbox()
        }
    }

    impl BoundingBoxHelpers for Collider {
        fn get_bounding_box(&self) -> Rect {
            self.rect
        }
    }

    #[test]
    fn test_no_colliders_result_in_no_displacement() {
        let results = simple_collision_resolution(&mut make_simple_10x10_entity(), vec![]);
        assert!(!results.was_displaced);
    }

    #[test]
    fn test_entites_are_displaced_leftward() {
        let mut entity = make_simple_10x10_entity();
        let collider = make_simple_collider(entity.offset_right_by(1));
        let results = simple_collision_resolution(&mut entity, vec![collider]);
        assert!(results.was_displaced);
        assert!(entity.is_just_left_of(collider));
    }

    #[test]
    fn test_entites_are_displaced_horizontally_when_vertically_smooshed() {
        let mut entity = make_simple_10x10_entity().with_previous_velocity(Vec2::new(1., 0.));
        let top_collider = make_simple_collider(entity.offset_up_by(11));
        let bottom_collider =
            make_simple_dynamic_collider(entity.offset_down_by(9), Vec2::new(0., -2.));
        let results = simple_collision_resolution(&mut entity, vec![top_collider, bottom_collider]);
        assert!(results.was_displaced);
        assert!(!results.is_penetrating_collider);
        assert!(entity.is_just_left_of(top_collider));
    }
}
