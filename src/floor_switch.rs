use macroquad::prelude::{Rect, BLACK, WHITE};

use crate::{
    config::config,
    entity::{Entity, EntityMap},
    math_util::contract_rect,
    physics::PhysicsComponent,
    sprite_component::{Renderer, SpriteComponent},
    switch::SwitchComponent,
};

pub struct FloorSwitchComponent();

pub fn create_floor_switch(start_rect: Rect, trigger_entity_iid: Option<&'static str>) -> Entity {
    let start_point = start_rect.point();
    let mut relative_bbox = start_rect.offset(-start_point);
    let drawn_rect = contract_rect(&relative_bbox, config().sprite_scale * 4.);
    relative_bbox.y -= config().sprite_scale * 2.;
    return Entity {
        sprite: SpriteComponent {
            pos: start_point,
            relative_bbox,
            renderer: Renderer::SolidRectangle(drawn_rect),
            color: Some(BLACK),
            ..Default::default()
        },
        physics: PhysicsComponent {
            defies_gravity: true,
            ..Default::default()
        },
        floor_switch: Some(FloorSwitchComponent()),
        switch: Some(SwitchComponent {
            trigger_entity_iid,
            ..Default::default()
        }),
        ..Default::default()
    };
}

pub fn floor_switch_system(entities: &mut EntityMap) {
    for entity in entities.values_mut() {
        if entity.floor_switch.is_some() {
            let color = if entity.switch.as_ref().unwrap().is_switched_on {
                WHITE
            } else {
                BLACK
            };
            entity.sprite.color = Some(color);
        }
    }
}
