use bitflags::bitflags;
use macroquad::prelude::*;

use crate::math_util::are_opposites;

#[derive(Debug)]
pub struct Collision {
    pub side: Side,
    pub displacement: Vec2,
}

bitflags! {
    pub struct CollisionFlags: u32 {
        const ENVIRONMENT = 0b00000001;
        const PLAYER_ONLY = 0b00000010;
    }
}

impl Default for CollisionFlags {
    fn default() -> Self {
        CollisionFlags::ENVIRONMENT
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Collider {
    pub rect: Rect,
    pub prev_rect: Rect,
    pub flags: CollisionFlags,
    pub entity_id: Option<u64>,
    pub enable_top: bool,
    pub enable_bottom: bool,
    pub enable_right: bool,
    pub enable_left: bool,
    pub velocity: Vec2,
}

#[derive(Debug, PartialEq)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

/// If we have more than this many displacements for a single entity while performing
/// collision resolution, assume we're in an infinite displacement loop and abort.
const MAX_DISPLACEMENTS_PER_FRAME: u32 = 30;

/// A number very close to zero that we add to all our displacements during collision
/// resolution, to ensure that we always displace entities outside of whatever they're
/// colliding with, irrespective of the vagaries of floating point arithmetic.
const EXTRA_DISPLACEMENT: f32 = 0.001;

impl Collider {
    pub fn draw_debug_rect(&self, color: Color) {
        let thickness = 2.;
        if self.enable_top {
            draw_line(
                self.rect.left(),
                self.rect.top(),
                self.rect.right(),
                self.rect.top(),
                thickness,
                color,
            );
        }
        if self.enable_bottom {
            draw_line(
                self.rect.left(),
                self.rect.bottom(),
                self.rect.right(),
                self.rect.bottom(),
                thickness,
                color,
            );
        }
        if self.enable_left {
            draw_line(
                self.rect.left(),
                self.rect.top(),
                self.rect.left(),
                self.rect.bottom(),
                thickness,
                color,
            );
        }
        if self.enable_right {
            draw_line(
                self.rect.right(),
                self.rect.top(),
                self.rect.right(),
                self.rect.bottom(),
                thickness,
                color,
            );
        }
    }
}

pub fn process_collision(
    collider: &Collider,
    actor_prev_bbox: &Rect,
    actor_bbox: &Rect,
    vertical_collision_leeway: f32,
) -> Option<Collision> {
    let collider_rect = collider.rect;

    if let Some(intersection) = collider_rect.intersect(*actor_bbox) {
        if collider.enable_top
            && intersection.top() <= collider_rect.top()
            // Make sure the actor was above our collider in the last frame.
            // Without this check, the actor can "pop" to the top side of
            // a collider instead of hitting its side.
            && actor_prev_bbox.bottom() <= collider.prev_rect.top()
        {
            // The top of the collider is being intersected with.
            let y_diff = actor_bbox.bottom() - collider_rect.top();
            return Some(Collision {
                side: Side::Top,
                displacement: Vec2::new(0., -y_diff - EXTRA_DISPLACEMENT),
            });
        } else if collider.enable_bottom
            && intersection.bottom() >= collider_rect.bottom()
            // Make sure the actor was below our collider in the last frame.
            // Without this check, the actor can "pop" to the bottom side
            // of a collider instead of hitting its side.
            && actor_prev_bbox.top() >= collider.prev_rect.bottom()
        {
            // The bottom side of the collider is being intersected with.
            let y_diff = collider_rect.bottom() - actor_bbox.top();
            return Some(Collision {
                side: Side::Bottom,
                displacement: Vec2::new(0., y_diff + EXTRA_DISPLACEMENT),
            });
        } else {
            let left_or_right_collision =
                if collider.enable_left && intersection.left() <= collider_rect.left() {
                    // The left side of the collider is being intersected with.
                    let x_diff = actor_bbox.right() - collider_rect.left();
                    Some(Collision {
                        side: Side::Left,
                        displacement: Vec2::new(-x_diff - EXTRA_DISPLACEMENT, 0.),
                    })
                } else if collider.enable_right && intersection.right() >= collider_rect.right() {
                    // The right side of the collider is being intersected with.
                    let x_diff = collider_rect.right() - actor_bbox.left();
                    Some(Collision {
                        side: Side::Right,
                        displacement: Vec2::new(x_diff + EXTRA_DISPLACEMENT, 0.),
                    })
                } else {
                    None
                };
            if left_or_right_collision.is_some() {
                let has_top_collision =
                    collider.enable_top && intersection.top() <= collider_rect.top();
                let is_actor_not_moving_up = actor_prev_bbox.y <= actor_bbox.y;
                if has_top_collision
                    && intersection.h <= vertical_collision_leeway
                    && is_actor_not_moving_up
                {
                    // We're colliding with the left or right side of the collider, but
                    // just barely, and we're *also* colliding with the top side of it,
                    // and the actor isn't on their way up.
                    //
                    // Instead of displacing the actor to the left or right of the collider,
                    // let's put them on top of it--this will ensure that the actor won't
                    // confusingly bump up against the sides of two adjacent colliders
                    // immediately below them, and it will also allow them to traverse
                    // very small gaps without needing to jump.
                    let y_diff = actor_bbox.bottom() - collider_rect.top();
                    return Some(Collision {
                        side: Side::Top,
                        displacement: Vec2::new(0., -y_diff - EXTRA_DISPLACEMENT),
                    });
                }
                return left_or_right_collision;
            }
        }
    }

    None
}

pub struct CollisionResolutionResult {
    pub displacements: u32,
    pub aborted: bool,
}

pub fn collision_resolution_loop<F: FnMut(u32) -> bool>(
    mut resolve_collisions: F,
) -> CollisionResolutionResult {
    let mut displacements = 0;

    loop {
        let displacement_occurred = resolve_collisions(displacements);
        if !displacement_occurred {
            return CollisionResolutionResult {
                displacements,
                aborted: false,
            };
        }
        displacements += 1;
        if displacements > MAX_DISPLACEMENTS_PER_FRAME {
            return CollisionResolutionResult {
                displacements,
                aborted: true,
            };
        }
    }
}

pub fn maybe_reverse_direction_x(velocity: &mut Vec2, displacement: &Vec2) {
    if are_opposites(displacement.x, velocity.x) {
        velocity.x = -velocity.x;
    }
}

pub fn maybe_reverse_direction_xy(velocity: &mut Vec2, displacement: &Vec2) {
    maybe_reverse_direction_x(velocity, displacement);
    if are_opposites(displacement.y, velocity.y) {
        velocity.y = -velocity.y;
    }
}
