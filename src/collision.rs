use macroquad::prelude::*;

use crate::math_util::are_opposites;

pub struct Collision {
    pub side: Side,
    pub displacement: Vec2,
}

#[derive(Default, Clone, Copy)]
pub struct Collider {
    pub rect: Rect,
    pub prev_rect: Rect,
    pub entity_id: Option<u64>,
    pub enable_top: bool,
    pub enable_bottom: bool,
    pub enable_right: bool,
    pub enable_left: bool,
    pub velocity: Vec2,
}

#[derive(PartialEq)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

const MAX_DISPLACEMENTS_PER_FRAME: u32 = 30;

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
                displacement: Vec2::new(0., -y_diff),
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
                displacement: Vec2::new(0., y_diff),
            });
        } else {
            let left_or_right_collision =
                if collider.enable_left && intersection.left() <= collider_rect.left() {
                    // The left side of the collider is being intersected with.
                    let x_diff = actor_bbox.right() - collider_rect.left();
                    Some(Collision {
                        side: Side::Left,
                        displacement: Vec2::new(-x_diff, 0.),
                    })
                } else if collider.enable_right && intersection.right() >= collider_rect.right() {
                    // The right side of the collider is being intersected with.
                    let x_diff = collider_rect.right() - actor_bbox.left();
                    Some(Collision {
                        side: Side::Right,
                        displacement: Vec2::new(x_diff, 0.),
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
                        displacement: Vec2::new(0., -y_diff),
                    });
                }
                return left_or_right_collision;
            }
        }
    }

    None
}

pub fn collision_resolution_loop<F: FnMut() -> bool>(mut resolve_collisions: F) {
    let mut displacements_this_frame = 0;

    loop {
        let displacement_occurred = resolve_collisions();
        if !displacement_occurred {
            break;
        }
        displacements_this_frame += 1;
        if displacements_this_frame > MAX_DISPLACEMENTS_PER_FRAME {
            println!(
                "WARNING: stuck in possible displacement loop, aborting collision resolution."
            );
            break;
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
