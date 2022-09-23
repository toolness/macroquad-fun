use macroquad::prelude::*;

use crate::math_util::are_opposites;

pub struct Collision {
    pub side: Side,
    pub displacement: Vec2,
}

#[derive(Default, Clone, Copy)]
pub struct Collider {
    pub rect: Rect,
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
) -> Option<Collision> {
    let collider_rect = collider.rect;

    if let Some(intersection) = collider_rect.intersect(*actor_bbox) {
        if collider.enable_top
            && intersection.top() <= collider_rect.top()
            && actor_prev_bbox.bottom() <= collider_rect.top()
        {
            // The top of the collider is being intersected with.
            let y_diff = actor_bbox.bottom() - collider_rect.top();
            return Some(Collision {
                side: Side::Top,
                displacement: Vec2::new(0., -y_diff),
            });
        } else if collider.enable_bottom
            && intersection.bottom() >= collider_rect.bottom()
            && actor_prev_bbox.top() >= collider_rect.bottom()
        {
            // The bottom side of the collider is being intersected with.
            let y_diff = collider_rect.bottom() - actor_bbox.top();
            return Some(Collision {
                side: Side::Bottom,
                displacement: Vec2::new(0., y_diff),
            });
        } else if collider.enable_left && intersection.left() <= collider_rect.left() {
            // The left side of the collider is being intersected with.
            let x_diff = actor_bbox.right() - collider_rect.left();
            return Some(Collision {
                side: Side::Left,
                displacement: Vec2::new(-x_diff, 0.),
            });
        } else if collider.enable_right && intersection.right() >= collider_rect.right() {
            // The right side of the collider is being intersected with.
            let x_diff = collider_rect.right() - actor_bbox.left();
            return Some(Collision {
                side: Side::Right,
                displacement: Vec2::new(x_diff, 0.),
            });
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
