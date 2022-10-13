use macroquad::{
    prelude::{Vec2, YELLOW},
    shapes::draw_line,
};

use crate::entity::{Entity, EntityMap, EntityProcessor};

pub struct RouteComponent {
    pub start_point: Vec2,
    pub end_point: Vec2,
    pub is_moving_towards_start: bool,
    pub can_be_blocked: bool,
    pub is_moving: bool,
    pub ping_pong: bool,
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

pub struct RouteSystem {
    pub processor: EntityProcessor,
}

impl RouteSystem {
    pub fn run(&mut self, entities: &mut EntityMap) {
        self.processor.filter_and_process_entities(
            entities,
            |entity| entity.route.is_some(),
            |entity, _entities| {
                let route = entity.route.as_mut().unwrap();
                if !route.is_moving {
                    return;
                }
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
                        if route.ping_pong {
                            route.is_moving_towards_start = !route.is_moving_towards_start;
                        } else {
                            route.is_moving = false;
                        }
                    }
                }
            },
        );
    }
}

pub fn try_to_start_route(entity: &mut Entity, move_towards_start: bool) -> bool {
    if let Some(route) = entity.route.as_mut() {
        route.is_moving = true;
        entity.physics.velocity = Vec2::ZERO;
        route.is_moving_towards_start = move_towards_start;
        true
    } else {
        false
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
