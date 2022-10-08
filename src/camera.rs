use macroquad::{
    prelude::{set_camera, Camera2D, Rect, Vec2, BLUE},
    window::{screen_height, screen_width},
};

use crate::{
    config::config,
    drawing::{draw_crosshair, draw_rect_lines},
    level::Level,
    math_util::{contract_rect_xy, floor_rect},
    sprite_component::SpriteComponent,
    time::GameTime,
};

#[derive(Default)]
pub struct Camera {
    current_rect: Rect,
    is_panning_next_update: bool,
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
            x_axis: Default::default(),
            y_axis: Default::default(),
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

    pub fn update(&mut self, sprite: &SpriteComponent, level: &Level, time: &GameTime) {
        let bbox = sprite.bbox();
        let deadzone_rect = self.get_deadzone_rect();
        let mut target = Vec2::new(bbox.x + bbox.w / 2., bbox.y + bbox.h / 2.);
        target.x += if sprite.is_facing_left {
            -self.current_rect.w * self.facing_offset_percentage
        } else {
            self.current_rect.w * self.facing_offset_percentage
        };
        self.target = target;
        let target_rect = calculate_camera_rect(
            &self.target,
            &level.pixel_bounds(),
            self.current_rect.w,
            self.current_rect.h,
        );

        let time_since_last_frame = time.time_since_last_frame as f32;
        if self.is_panning_next_update {
            self.x_axis.update(
                target_rect.x,
                self.target.x >= deadzone_rect.left() && self.target.x <= deadzone_rect.right(),
                time_since_last_frame,
            );
            self.y_axis.update(
                target_rect.y,
                self.target.y >= deadzone_rect.top() && self.target.y <= deadzone_rect.bottom(),
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
        draw_rect_lines(&self.get_deadzone_rect(), 2., BLUE);
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
    pos: f32,
    velocity: f32,
}

impl CameraAxis {
    fn reset(&mut self, pos: f32) {
        self.pos = pos;
        self.velocity = 0.;
    }

    fn update(&mut self, target: f32, is_target_in_deadzone: bool, time_since_last_frame: f32) {
        let acceleration = config().camera_acceleration;
        let to_target = target - self.pos;
        let direction_to_target = if to_target < 0. { -1. } else { 1. };
        let direction_from_target = -1. * direction_to_target;
        let has_reached_target = to_target.abs() < 1.;
        if is_target_in_deadzone || has_reached_target {
            self.velocity += direction_from_target * acceleration * time_since_last_frame;
            if self.velocity * direction_to_target <= 0. {
                // We are now going in the opposite direction, but we don't want to do that, so stop.
                self.velocity = 0.
            }
        } else {
            self.velocity += direction_to_target * acceleration * time_since_last_frame;
        }

        if !has_reached_target {
            self.pos += self.velocity * time_since_last_frame;
            let new_to_target = target - self.pos;
            let is_moving_towards_target = to_target * new_to_target > 0.;
            if !is_moving_towards_target {
                // We just overshot the target!
                self.pos = target;
            }
        }
    }
}
