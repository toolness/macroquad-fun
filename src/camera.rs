use macroquad::{
    prelude::{set_camera, Camera2D, Rect, Vec2, BLUE},
    window::{screen_height, screen_width},
};

use crate::{
    config::config,
    drawing::{draw_crosshair, draw_rect_lines},
    entity::Entity,
    level::Level,
    math_util::{contract_rect_xy, floor_rect},
    time::GameTime,
};

#[derive(Default)]
pub struct Camera {
    current_rect: Rect,
    is_panning_next_update: bool,
    is_centered_without_deadzone: bool,
    x_axis: CameraAxis,
    y_axis: CameraAxis,
    deadzone_percentage: Vec2,
    facing_offset_percentage: f32,
    target: Vec2,
}

impl Camera {
    pub fn new() -> Self {
        let config = config();
        Camera {
            current_rect: Rect::new(0., 0., screen_width(), screen_height()),
            is_panning_next_update: false,
            x_axis: CameraAxis::new("X"),
            y_axis: CameraAxis::new("Y"),
            is_centered_without_deadzone: false,
            deadzone_percentage: Vec2::new(
                config.camera_deadzone_width_percentage,
                config.camera_deadzone_height_percentage,
            ),
            facing_offset_percentage: config.camera_facing_offset_percentage,
            target: Default::default(),
        }
    }

    fn get_deadzone_rect(&self) -> Rect {
        let w = self.current_rect.w;
        let h = self.current_rect.h;
        let width_reduction = w - (w * self.deadzone_percentage.x);
        let height_reduction = h - (h * self.deadzone_percentage.y);
        contract_rect_xy(
            &self.current_rect,
            width_reduction / 2.,
            height_reduction / 2.,
        )
    }

    fn should_center_without_deadzone(&self, entity: &Entity) -> bool {
        if config().camera_disable_deadzone {
            return true;
        }
        let is_attached = entity
            .attachment
            .as_ref()
            .map(|a| a.is_attached())
            .unwrap_or(false);
        is_attached || entity.physics.latest_frame.is_on_moving_surface
    }

    pub fn update(&mut self, entity: &Entity, level: &Level, time: &GameTime) {
        let sprite = &entity.sprite;
        let bbox = sprite.bbox();
        let deadzone_rect = self.get_deadzone_rect();
        self.is_centered_without_deadzone = self.should_center_without_deadzone(entity);
        let mut target = Vec2::new(bbox.x + bbox.w / 2., bbox.y + bbox.h / 2.);
        if !self.is_centered_without_deadzone {
            target.x += if sprite.is_facing_left {
                -self.current_rect.w * self.facing_offset_percentage
            } else {
                self.current_rect.w * self.facing_offset_percentage
            };
        }
        self.target = target;
        let level_rect = level.pixel_bounds();
        let target_rect = calculate_camera_rect(
            &self.target,
            &level_rect,
            self.current_rect.w,
            self.current_rect.h,
        );

        let time_since_last_frame = time.time_since_last_frame as f32;
        if self.is_panning_next_update {
            self.x_axis.update(
                target_rect.x,
                !self.is_centered_without_deadzone
                    && self.target.x >= deadzone_rect.left()
                    && self.target.x <= deadzone_rect.right(),
                time_since_last_frame,
            );
            self.y_axis.update(
                target_rect.y,
                !self.is_centered_without_deadzone
                    && self.target.y >= deadzone_rect.top()
                    && self.target.y <= deadzone_rect.bottom(),
                time_since_last_frame,
            );
            self.current_rect.x = self.x_axis.pos;
            self.current_rect.y = self.y_axis.pos;
        } else {
            self.current_rect = target_rect;
            self.x_axis.reset(target_rect.x);
            self.y_axis.reset(target_rect.y);
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

    pub fn draw_debug_info(&self) {
        if !self.is_centered_without_deadzone {
            draw_rect_lines(&self.get_deadzone_rect(), 2., BLUE);
        }
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

#[derive(Default, Debug)]
struct CameraAxis {
    #[allow(dead_code)]
    name: &'static str,
    pos: f32,
    target: f32,
    start_pos: f32,
    time_elapsed: f32,
}

impl CameraAxis {
    fn new(name: &'static str) -> Self {
        CameraAxis {
            name,
            ..Default::default()
        }
    }

    fn reset(&mut self, pos: f32) {
        self.pos = pos;
        self.target = self.pos;
        self.start_pos = pos;
    }

    fn update(&mut self, target: f32, is_target_in_deadzone: bool, time_since_last_frame: f32) {
        let total_time = config().camera_ms_time_to_target / 1000.;
        if self.target != target {
            self.time_elapsed = 0.;
            self.start_pos = self.pos;
            self.target = target;
        }
        self.time_elapsed += time_since_last_frame;
        if self.time_elapsed > total_time {
            self.time_elapsed = total_time;
        }
        self.pos = self.start_pos
            + ease_in_out(self.time_elapsed / total_time) * (self.target - self.start_pos);
    }
}

fn ease_in_out(x: f32) -> f32 {
    // https://easings.net/#easeInOutQuad
    if x < 0.5 {
        2. * x * x
    } else {
        1. - (-2. * x + 2.).powi(2) / 2.
    }
}
