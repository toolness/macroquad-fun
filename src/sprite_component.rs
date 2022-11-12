use macroquad::{
    prelude::{Color, Rect, Vec2, GREEN, PURPLE, WHITE},
    shapes::draw_rectangle,
};

use crate::{
    drawing::draw_rect_lines,
    level::Level,
    materials::MaterialRenderer,
    sprite_renderer::{SpriteDrawParams, SpriteRenderer},
    time::GameTime,
};

#[derive(Clone, Copy)]
pub enum Rotation {
    None,
    Clockwise270,
}

impl Default for Rotation {
    fn default() -> Self {
        Rotation::None
    }
}

#[derive(Default, Clone, Copy)]
pub enum Renderer {
    #[default]
    None,
    SolidRectangle(Rect),
    Sprite(&'static SpriteRenderer),
    EntityTiles(Rect),
}

#[derive(Default, Clone, Copy)]
pub struct SpriteComponent {
    pub pos: Vec2,

    /// The bounding box of the sprite relative to the top-left of a
    /// single frame of its renderer, without any flipping or rotation
    /// applied.
    ///
    /// This is only public for ease of construction. Prefer
    /// `relative_bbox()` to this.
    pub base_relative_bbox: Rect,

    pub rotation: Rotation,
    pub renderer: Renderer,
    pub material: MaterialRenderer,
    pub color: Option<Color>,
    pub is_facing_left: bool,
    pub left_facing_rendering: LeftFacingRendering,
    pub current_frame_number: u32,
}

#[derive(Default, Clone, Copy)]
/// Rendering/bounding box behavior of a sprite component when it's facing
/// left (by default, we assume it's facing right).
///
/// We always horizontally flip the sprite itself, but any extra behavior
/// is defined by this enum.
pub enum LeftFacingRendering {
    #[default]
    /// Just horizontally flip the sprite, don't change its bounding box.
    Default,

    /// Don't change the sprite's bounding box (as this could result in weird
    /// physics bugs); instead, shift the x-coordinate of where we render the
    /// sprite by this amount.
    XOffset(f32),

    /// Horizontally flip the sprite's bounding box too.
    FlipBoundingBox,
}

impl SpriteComponent {
    fn calculate_relative_bbox(&self, relative_bbox: &Rect) -> Rect {
        let mut bbox = *relative_bbox;
        match self.rotation {
            Rotation::None => {}
            Rotation::Clockwise270 => {
                bbox = Rect::new(bbox.y, bbox.x, bbox.h, bbox.w);
            }
        }
        if self.is_facing_left {
            match self.left_facing_rendering {
                LeftFacingRendering::Default => {}

                // Note that we're going to keep the bounding box the same here--the x-offset
                // is used at *render* time, not to calculate the bounding box.
                LeftFacingRendering::XOffset(..) => {}

                LeftFacingRendering::FlipBoundingBox => {
                    if let Renderer::Sprite(sprite) = self.renderer {
                        let center_offset = sprite.frame_width() / 2. - bbox.w / 2.;
                        let flipped_x = (bbox.x - center_offset) * -1. + center_offset;
                        bbox.x = flipped_x;
                    }
                }
            }
        }
        bbox
    }

    pub fn relative_bbox(&self) -> Rect {
        self.calculate_relative_bbox(&self.base_relative_bbox)
    }

    pub fn calculate_absolute_bounding_box(&self, relative_bbox: &Rect) -> Rect {
        self.calculate_relative_bbox(relative_bbox).offset(self.pos)
    }

    pub fn bbox(&self) -> Rect {
        self.calculate_absolute_bounding_box(&self.base_relative_bbox)
    }

    pub fn at_bottom_left(mut self, rect: &Rect) -> Self {
        let relative_bbox = self.relative_bbox();
        self.pos.x = rect.left() - relative_bbox.left();
        self.pos.y = rect.bottom() - relative_bbox.bottom();
        self
    }

    pub fn at_top_left(mut self, rect: &Rect) -> Self {
        let relative_bbox = self.relative_bbox();
        self.pos.x = rect.left() - relative_bbox.left();
        self.pos.y = rect.top() - relative_bbox.top();
        self
    }

    pub fn update_looping_frame_number(&mut self, time: &GameTime) {
        if let Renderer::Sprite(sprite) = self.renderer {
            self.current_frame_number = time.looping_frame_number(&sprite);
        }
    }

    fn get_sprite_x(&self) -> f32 {
        let mut x = self.pos.x;
        if self.is_facing_left {
            if let LeftFacingRendering::XOffset(offset) = self.left_facing_rendering {
                x += offset;
            }
        }
        x
    }

    pub fn draw_current_frame(&self, level: &Level) {
        self.material.start_using();
        match self.renderer {
            Renderer::None => {}
            Renderer::Sprite(sprite) => {
                sprite.draw_ex(
                    self.get_sprite_x(),
                    self.pos.y,
                    self.current_frame_number,
                    SpriteDrawParams {
                        flip_x: self.is_facing_left,
                        rotation: match self.rotation {
                            Rotation::None => 0.,
                            Rotation::Clockwise270 => -std::f64::consts::FRAC_PI_2 as f32,
                        },
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
        self.material.stop_using();
    }

    pub fn draw_debug_rects(&self) {
        if let Renderer::Sprite(sprite) = self.renderer {
            sprite.draw_debug_rect(self.pos.x, self.pos.y, GREEN);
        }
        draw_rect_lines(&self.bbox(), 2., PURPLE);
    }
}
