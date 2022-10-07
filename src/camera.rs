use macroquad::{
    prelude::{set_camera, Camera2D, Rect, Vec2},
    window::{screen_height, screen_width},
};

use crate::{
    config::config, level::Level, math_util::floor_rect, sprite_component::SpriteComponent,
    time::GameTime,
};

#[derive(Default)]
pub struct Camera {
    current_rect: Rect,
    is_panning_next_update: bool,
    velocity: f32,
    acceleration: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            current_rect: Default::default(),
            is_panning_next_update: false,
            velocity: 0.,
            acceleration: config().camera_acceleration,
        }
    }

    pub fn update(&mut self, sprite: &SpriteComponent, level: &Level, time: &GameTime) {
        let bbox = sprite.bbox();
        let bbox_center = Vec2::new(bbox.x + bbox.w / 2., bbox.y + bbox.h / 2.);
        let target_rect = calculate_camera_rect(&bbox_center, &level.pixel_bounds());
        let time_since_last_frame = time.time_since_last_frame as f32;
        if self.is_panning_next_update {
            let target = target_rect.point();
            let vector_to_target = target - self.current_rect.point();
            if vector_to_target.length_squared() < 1. {
                self.velocity -= self.acceleration * time_since_last_frame;
                if self.velocity < 0. {
                    self.velocity = 0.
                }
            } else {
                self.velocity += self.acceleration * time_since_last_frame;
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
