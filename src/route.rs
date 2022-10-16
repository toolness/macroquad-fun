use macroquad::{
    prelude::{Rect, Vec2, PURPLE, YELLOW},
    shapes::draw_line,
};

use crate::{
    config::config,
    drawing::draw_rect_lines,
    entity::{Entity, EntityMap, EntityProcessor},
    sprite_component::SpriteComponent,
};

pub struct RouteComponent {
    pub start_point: Vec2,
    pub end_point: Vec2,
    pub is_moving_towards_start: bool,
    pub stop_when_blocked: bool,
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
            |entity, entities| {
                let route = entity.route.as_mut().unwrap();
                if !route.is_moving {
                    return;
                }
                if route.stop_when_blocked && is_route_blocked(&route, &entity.sprite, entities) {
                    entity.physics.velocity = Vec2::ZERO;
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

fn is_route_blocked(
    route: &RouteComponent,
    sprite: &SpriteComponent,
    entities: &EntityMap,
) -> bool {
    if let Some(edge_bbox) = get_route_edge_bbox(route, sprite) {
        for (_id, entity) in entities.iter() {
            if entity.sprite.bbox().overlaps(&edge_bbox) {
                return true;
            }
        }
    }
    false
}

fn get_route_edge_bbox(route: &RouteComponent, sprite: &SpriteComponent) -> Option<Rect> {
    let thickness = config().blocked_route_edge_thickness;
    let bbox = sprite.bbox();
    let target = route.target();
    let direction_to_target = target - sprite.pos;

    if direction_to_target == Vec2::ZERO {
        None
    } else if direction_to_target.x != 0. {
        let x = if direction_to_target.x > 0. {
            bbox.right()
        } else {
            bbox.left() - thickness
        };
        Some(Rect::new(x, bbox.y, thickness, bbox.h))
    } else {
        let y = if direction_to_target.y > 0. {
            bbox.bottom()
        } else {
            bbox.top() - thickness
        };
        Some(Rect::new(bbox.x, y, bbox.w, thickness))
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
    for (_id, entity) in entities.iter() {
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
            if route.stop_when_blocked {
                if let Some(edge_bbox) = get_route_edge_bbox(&route, &entity.sprite) {
                    draw_rect_lines(&edge_bbox, 1., PURPLE)
                }
            }
        }
    }
}
