use macroquad::prelude::*;

pub struct Actor {
    pub prev_bbox: Rect,
    pub bbox: Rect,
    pub velocity: Vec2,
}

pub struct Collision {
    pub is_on_surface: bool,
    pub displacement: Vec2,
    pub new_velocity: Option<Vec2>,
}

pub fn process_collision(collider_rect: &Rect, actor: &Actor) -> Option<Collision> {
    let player_bbox = actor.bbox;
    let player_prev_bbox = actor.prev_bbox;

    if let Some(intersection) = collider_rect.intersect(player_bbox) {
        if intersection.top() <= collider_rect.top()
            && player_prev_bbox.bottom() <= collider_rect.top()
        {
            // The top of the collider is being intersected with.
            let y_diff = player_bbox.bottom() - collider_rect.top();
            return Some(Collision {
                is_on_surface: true,
                displacement: Vec2::new(0., -y_diff),
                new_velocity: Some(Vec2::new(0., 0.)),
            });
        } else if intersection.bottom() >= collider_rect.bottom()
            && player_prev_bbox.top() >= collider_rect.bottom()
        {
            // The bottom side of the collider is being intersected with.
            let y_diff = collider_rect.bottom() - player_bbox.top();
            return Some(Collision {
                is_on_surface: false,
                displacement: Vec2::new(0., y_diff),
                new_velocity: Some(Vec2::new(actor.velocity.x, 0.)),
            });
        } else if intersection.left() <= collider_rect.left() {
            // The left side of the collider is being intersected with.
            let x_diff = player_bbox.right() - collider_rect.left();
            return Some(Collision {
                is_on_surface: false,
                displacement: Vec2::new(-x_diff, 0.),
                new_velocity: None,
            });
        } else if intersection.right() >= collider_rect.right() {
            // The right side of the collider is being intersected with.
            let x_diff = collider_rect.right() - player_bbox.left();
            return Some(Collision {
                is_on_surface: false,
                displacement: Vec2::new(x_diff, 0.),
                new_velocity: None,
            });
        }
    }

    None
}
