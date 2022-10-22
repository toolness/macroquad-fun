use macroquad::{
    prelude::{
        gl_use_default_material, gl_use_material, Color, Material, Rect, Vec2, GREEN, PURPLE, WHITE,
    },
    shapes::draw_rectangle,
};

use crate::{
    drawing::draw_rect_lines,
    level::Level,
    sprite_renderer::{SpriteDrawParams, SpriteRenderer},
    time::GameTime,
};

#[derive(Default)]
pub enum Renderer {
    #[default]
    None,
    SolidRectangle(Rect),
    Sprite(&'static SpriteRenderer),
    EntityTiles(Rect),
}

#[derive(Default)]
pub struct SpriteComponent {
    pub pos: Vec2,
    pub relative_bbox: Rect,
    pub renderer: Renderer,
    pub material: Option<Material>,
    pub color: Option<Color>,
    pub is_facing_left: bool,
    pub flip_bbox_when_facing_left: bool,
    pub current_frame_number: u32,
}

impl SpriteComponent {
    pub fn calculate_absolute_bounding_box(&self, relative_bbox: &Rect) -> Rect {
        if self.flip_bbox_when_facing_left && self.is_facing_left {
            if let Renderer::Sprite(sprite) = self.renderer {
                let center_offset = sprite.frame_width() / 2. - relative_bbox.w / 2.;
                let flipped_x = (self.relative_bbox.x - center_offset) * -1. + center_offset;
                let mut flipped_relative_bbox = *relative_bbox;
                flipped_relative_bbox.x = flipped_x;
                return flipped_relative_bbox.offset(self.pos);
            }
        }
        relative_bbox.offset(self.pos)
    }

    pub fn bbox(&self) -> Rect {
        self.calculate_absolute_bounding_box(&self.relative_bbox)
    }

    pub fn at_bottom_left(mut self, rect: &Rect) -> Self {
        self.pos.x = rect.left() - self.relative_bbox.left();
        self.pos.y = rect.bottom() - self.relative_bbox.bottom();
        self
    }

    pub fn at_top_left(mut self, rect: &Rect) -> Self {
        self.pos.x = rect.left() - self.relative_bbox.left();
        self.pos.y = rect.top() - self.relative_bbox.top();
        self
    }

    pub fn update_looping_frame_number(&mut self, time: &GameTime) {
        if let Renderer::Sprite(sprite) = self.renderer {
            self.current_frame_number = time.looping_frame_number(&sprite);
        }
    }

    pub fn draw_current_frame(&self, level: &Level) {
        if let Some(material) = self.material {
            gl_use_material(material);
        }
        match self.renderer {
            Renderer::None => {}
            Renderer::Sprite(sprite) => {
                sprite.draw_ex(
                    self.pos.x,
                    self.pos.y,
                    self.current_frame_number,
                    SpriteDrawParams {
                        flip_x: self.is_facing_left,
                        color: self.color.unwrap_or(WHITE),
                        ..Default::default()
                    },
                );
            }
            Renderer::SolidRectangle(rect) => {
                if let Some(color) = self.color {
                    draw_rectangle(
                        self.pos.x + rect.x,
                        self.pos.y + rect.y,
                        rect.w,
                        rect.h,
                        color,
                    );
                }
            }
            Renderer::EntityTiles(rect) => level.draw_entity_tiles(&rect, &self.bbox().point()),
        }
        if self.material.is_some() {
            gl_use_default_material();
        }
    }

    pub fn draw_debug_rects(&self) {
        if let Renderer::Sprite(sprite) = self.renderer {
            sprite.draw_debug_rect(self.pos.x, self.pos.y, GREEN);
        }
        draw_rect_lines(&self.bbox(), 2., PURPLE);
    }
}
