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
    velocity: f32,
    acceleration: f32,
    deadzone_percentage: f32,
    facing_offset_percentage: f32,
    target: Vec2,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            current_rect: Rect::new(0., 0., screen_width(), screen_height()),
            is_panning_next_update: false,
            velocity: 0.,
            acceleration: config().camera_acceleration,
            deadzone_percentage: config().camera_deadzone_percentage,
            facing_offset_percentage: config().camera_facing_offset_percentage,
            target: Default::default(),
        }
    }

    fn get_deadzone_rect(&self) -> Rect {
        let w = self.current_rect.w;
        let h = self.current_rect.h;
        let width_reduction = w - (w * self.deadzone_percentage);
        let height_reduction = h - (h * self.deadzone_percentage);
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
        let is_target_inside_deadzone = deadzone_rect.contains(self.target);
        let target_rect = calculate_camera_rect(
            &self.target,
            &level.pixel_bounds(),
            self.current_rect.w,
            self.current_rect.h,
        );
        let time_since_last_frame = time.time_since_last_frame as f32;
        if self.is_panning_next_update {
            let target = target_rect.point();
            let vector_to_target = target - self.current_rect.point();
            let has_reached_target = vector_to_target.length_squared() < 1.;
            if is_target_inside_deadzone || has_reached_target {
                self.velocity -= self.acceleration * time_since_last_frame;
                if self.velocity < 0. {
                    self.velocity = 0.
                }
            } else {
                self.velocity += self.acceleration * time_since_last_frame;
            }
            if !has_reached_target {
                let velocity_vector = vector_to_target.normalize() * self.velocity;
                self.current_rect.x += velocity_vector.x * time_since_last_frame;
                self.current_rect.y += velocity_vector.y * time_since_last_frame;
                let new_vector_to_target = target - self.current_rect.point();
                let is_moving_towards_target = vector_to_target.dot(new_vector_to_target) > 0.;
                if !is_moving_towards_target {
                    // We just overshot the target!
                    self.current_rect.x = target.x;
                    self.current_rect.y = target.y;
                }
            }
        } else {
            self.velocity = 0.;
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
