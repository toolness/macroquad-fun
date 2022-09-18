use macroquad::prelude::Vec2;

use crate::{
    collision::{
        collision_resolution_loop, maybe_reverse_direction_x, maybe_reverse_direction_xy,
        process_collision, Side,
    },
    config::config,
    entity::EntityMap,
    level::{BoundsColliderIterator, Level},
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
    pub x_impulse: f32,
    pub prev_x_impulse: f32,
    pub defies_gravity: bool,
    pub defies_level_bounds: bool,
    pub collision_behavior: PhysicsCollisionBehavior,
    pub is_on_any_surface_this_frame: bool,
    pub was_displaced_this_frame: bool,
}

pub fn physics_system(entities: &mut EntityMap, level: &Level, time: &GameTime) {
    let gravity = config().gravity;
    let time_since_last_frame = time.time_since_last_frame as f32;
    let gravity_this_frame = gravity * time_since_last_frame;

    for (_id, entity) in entities.iter_mut() {
        let physics = &mut entity.physics;
        let sprite = &mut entity.sprite;

        if !physics.defies_gravity {
            physics.velocity.y += gravity_this_frame;
        }

        let prev_bbox = sprite.bbox();
        let mut is_on_any_surface_this_frame = false;
        let mut was_displaced_this_frame = false;

        sprite.pos += physics.velocity * time_since_last_frame;
        sprite.pos.x += physics.x_impulse * time_since_last_frame;
        physics.prev_x_impulse = physics.x_impulse;
        physics.x_impulse = 0.;

        collision_resolution_loop(|| {
            let bbox = sprite.bbox();

            let colliders = {
                let base = level.iter_colliders(&bbox);

                if physics.defies_level_bounds {
                    base.chain(BoundsColliderIterator::empty())
                } else {
                    base.chain(level.iter_bounds_as_colliders())
                }
            };

            for collider in colliders {
                if let Some(collision) = process_collision(&collider, &prev_bbox, &bbox) {
                    match collision.side {
                        Side::Top => {
                            is_on_any_surface_this_frame = true;
                            if !physics.defies_gravity {
                                physics.velocity.y = 0.;
                            }
                            if physics.collision_behavior == PhysicsCollisionBehavior::Stop {
                                physics.velocity.x = 0.;
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
                        was_displaced_this_frame = true;
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

        physics.is_on_any_surface_this_frame = is_on_any_surface_this_frame;
        physics.was_displaced_this_frame = was_displaced_this_frame;
    }
}
