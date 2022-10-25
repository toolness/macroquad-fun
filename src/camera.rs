use macroquad::{
    prelude::{pop_camera_state, push_camera_state, set_camera, Camera2D, Rect, Vec2, BLUE},
    window::{screen_height, screen_width},
};

use crate::{drawing::draw_crosshair, entity::Entity, level::Level, math_util::floor_rect};

#[derive(Default)]
pub struct Camera {
    current_rect: Rect,
    target: Vec2,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            current_rect: Rect::new(0., 0., screen_width(), screen_height()),
            target: Default::default(),
        }
    }

    pub fn with_active<F: FnOnce() -> ()>(&self, cb: F) {
        self.activate();
        cb();
        self.deactivate();
    }

    fn activate(&self) {
        push_camera_state();
        // Clamp to integers to avoid weird visual artifacts.
        let int_rect = floor_rect(&self.current_rect);
        set_camera(&Camera2D::from_display_rect(int_rect));
    }

    fn deactivate(&self) {
        pop_camera_state();
    }

    pub fn update(&mut self, entity: &Entity, level: &Level) {
        let sprite = &entity.sprite;
        let bbox = sprite.bbox();
        self.target = Vec2::new(bbox.x + bbox.w / 2., bbox.y + bbox.h / 2.);
        let target_rect = calculate_camera_rect(
            &self.target,
            &level.pixel_bounds(),
            self.current_rect.w,
            self.current_rect.h,
        );

        self.current_rect = target_rect;
    }

    pub fn rect(&self) -> &Rect {
        &self.current_rect
    }

    pub fn draw_debug_info(&self) {
        draw_crosshair(&self.target, 5., 1., BLUE);
    }
}

fn calculate_camera_rect(
    center: &Vec2,
    level_rect: &Rect,
    screen_width: f32,
    screen_height: f32,
) -> Rect {
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
