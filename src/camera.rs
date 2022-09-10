use macroquad::prelude::{set_camera, Camera2D, Rect, Vec2};

use crate::{config::config, level::Level, player::Player};

fn calculate_camera_rect(center: &Vec2, level_rect: &Rect) -> Rect {
    let config = config();
    let mut camera_rect = Rect::new(
        center.x - config.screen_width / 2.,
        center.y - config.screen_height / 2.,
        config.screen_width,
        config.screen_height,
    );
    if camera_rect.left() < level_rect.left() || camera_rect.w > level_rect.w {
        camera_rect.x = level_rect.left();
    } else if camera_rect.right() > level_rect.right() {
        camera_rect.x = level_rect.right() - camera_rect.w;
    }
    if camera_rect.top() < level_rect.top() || camera_rect.h > level_rect.h {
        camera_rect.y = level_rect.top();
    } else if camera_rect.bottom() > level_rect.bottom() {
        camera_rect.y = level_rect.bottom() - camera_rect.h;
    }
    camera_rect
}

pub fn center_camera(player: &Player, level: &Level) -> Rect {
    let bbox = player.entity().bbox();
    let bbox_center = Vec2::new(bbox.x + bbox.w / 2., bbox.y + bbox.h / 2.);
    let camera_rect = calculate_camera_rect(&bbox_center, &level.pixel_bounds());
    set_camera(&Camera2D::from_display_rect(camera_rect));
    camera_rect
}
