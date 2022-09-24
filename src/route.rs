use macroquad::{
    prelude::{Vec2, YELLOW},
    shapes::draw_line,
};

use crate::entity::EntityMap;

pub struct RouteComponent {
    pub start_point: Vec2,
    pub end_point: Vec2,
    pub is_moving_towards_start: bool,
    pub speed: f32,
}

impl RouteComponent {
    fn target(&self) -> Vec2 {
        if self.is_moving_towards_start {
            self.start_point
        } else {
            self.end_point
        }
    }
}

pub fn route_system(entities: &mut EntityMap) {
    for entity in entities.values_mut() {
        if let Some(route) = entity.route.as_mut() {
            let target = route.target();
            let direction_to_target = target - entity.sprite.pos;
            if entity.physics.velocity == Vec2::ZERO {
                entity.physics.velocity = direction_to_target.normalize() * route.speed;
            } else {
                let is_moving_towards_target =
                    entity.physics.velocity.dot(direction_to_target) > 0.;
                if !is_moving_towards_target {
                    entity.sprite.pos = target;
                    entity.physics.velocity = Vec2::ZERO;
                    route.is_moving_towards_start = !route.is_moving_towards_start;
                }
            }
        }
    }
}

pub fn draw_route_debug_targets(entities: &EntityMap) {
    for entity in entities.values() {
        if let Some(route) = &entity.route {
            let target = route.target();
            draw_line(
                entity.sprite.pos.x,
                entity.sprite.pos.y,
                target.x,
                target.y,
                1.,
                YELLOW,
            );
        }
    }
}
