use macroquad::{
    prelude::{clamp, set_camera, Camera2D, Rect, Vec2},
    window::{screen_height, screen_width},
};

use crate::{
    config::config, level::Level, math_util::floor_rect, sprite_component::SpriteComponent,
};

#[derive(Default)]
pub struct Camera {
    current_rect: Rect,
    is_panning_next_update: bool,
    max_pan_speed: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            current_rect: Default::default(),
            is_panning_next_update: false,
            max_pan_speed: config().sprite_scale * 1.,
        }
    }

    pub fn update(&mut self, sprite: &SpriteComponent, level: &Level) {
        let bbox = sprite.bbox();
        let bbox_center = Vec2::new(bbox.x + bbox.w / 2., bbox.y + bbox.h / 2.);
        let target_rect = calculate_camera_rect(&bbox_center, &level.pixel_bounds());
        if self.is_panning_next_update {
            let delta = target_rect.point() - self.current_rect.point();
            self.current_rect.x += clamp(delta.x, -self.max_pan_speed, self.max_pan_speed);
            self.current_rect.y += clamp(delta.y, -self.max_pan_speed, self.max_pan_speed);
        } else {
            self.current_rect = target_rect;
            self.is_panning_next_update = true;
        }
        // Clamp to integers to avoid weird visual artifacts.
        let int_rect = floor_rect(&self.current_rect);
        set_camera(&Camera2D::from_display_rect(int_rect));
    }

    pub fn cut(&mut self) {
        self.is_panning_next_update = false;
    }

    pub fn rect(&self) -> &Rect {
        &self.current_rect
    }
}

fn calculate_camera_rect(center: &Vec2, level_rect: &Rect) -> Rect {
    let screen_width = screen_width();
    let screen_height = screen_height();
    let mut camera_rect = Rect::new(
        center.x - screen_width / 2.,
        center.y - screen_height / 2.,
        screen_width,
        screen_height,
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
