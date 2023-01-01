use macroquad::{
    prelude::{Color, Rect, Vec2, GREEN, PURPLE, WHITE},
    shapes::{draw_rectangle, draw_rectangle_lines},
};

use crate::{
    drawing::draw_rect_lines,
    level::Level,
    materials::MaterialRenderer,
    sprite_renderer::{SpriteDrawParams, SpriteRenderer},
    time::GameTime,
};

#[derive(Clone, Copy, Default)]
pub enum Rotation {
    #[default]
    None,
    Clockwise270,
}

#[derive(Default, Clone, Copy)]
pub enum Renderer {
    #[default]
    Sprite,
    Invisible,
    SolidRectangle(Rect),
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

    /// This is used to render the sprite, if the renderer is set to
    /// [Renderer::Sprite]. If it's not, the dimensions of this sprite
    /// can still be used in the computation of bounding boxes and such
    /// (this can be useful for entities that are children of parents).
    pub sprite: Option<&'static SpriteRenderer>,

    pub material: MaterialRenderer,

    /// The tint of the sprite, or the fill color of the shape, depending
    /// on which renderer is active.
    pub color: Option<Color>,

    pub is_facing_left: bool,
    pub left_facing_rendering: LeftFacingRendering,

    /// We sometimes don't want to change the sprite's bounding box when it
    /// is facing left, as this could result in weird physics bugs; instead,
    /// this can be used to shift the x-coordinate of where we render
    /// the sprite.
    ///
    /// In other words, this doesn't actually affect the sprite's bounding
    /// box at all--it just shifts where the sprite is rendered.
    pub left_facing_x_offset: f32,

    /// When the sprite is facing left, this shifts the sprite's bounding
    /// box by the given amount. It does *not* affect where the sprite is
    /// rendered.
    pub left_facing_bbox_x_offset: f32,

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

    /// Horizontally flip the sprite's bounding box too.
    FlipBoundingBox,
}

impl SpriteComponent {
    fn get_sprite_dimensions(&self, sprite: &SpriteRenderer) -> Vec2 {
        match self.rotation {
            Rotation::None => Vec2::new(sprite.frame_width(), sprite.frame_height()),
            Rotation::Clockwise270 => Vec2::new(sprite.frame_height(), sprite.frame_width()),
        }
    }

    fn calculate_relative_bbox(&self, relative_bbox: &Rect) -> Rect {
        let mut bbox = *relative_bbox;
        match self.rotation {
            Rotation::None => {}
            Rotation::Clockwise270 => {
                bbox = Rect::new(bbox.y, bbox.x, bbox.h, bbox.w);

                if let Some(sprite) = self.sprite {
                    let frame_height = self.get_sprite_dimensions(&sprite).y;
                    let center_offset = frame_height / 2. - bbox.h / 2.;
                    let flipped_y = (bbox.y - center_offset) * -1. + center_offset;
                    bbox.y = flipped_y;
                }
            }
        }
        if self.is_facing_left {
            match self.left_facing_rendering {
                LeftFacingRendering::Default => {}

                LeftFacingRendering::FlipBoundingBox => {
                    if let Some(sprite) = self.sprite {
                        let frame_width = self.get_sprite_dimensions(&sprite).x;
                        let center_offset = frame_width / 2. - bbox.w / 2.;
                        let flipped_x = (bbox.x - center_offset) * -1. + center_offset;
                        bbox.x = flipped_x + self.left_facing_bbox_x_offset;
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

    pub fn with_pos_and_size(mut self, rect: &Rect) -> Self {
        let start_point = rect.point();
        let relative_bbox = rect.offset(-start_point);
        self.pos = start_point;
        self.base_relative_bbox = relative_bbox;
        self
    }

    pub fn update_looping_frame_number(&mut self, time: &GameTime) {
        if let Some(sprite) = self.sprite {
            self.current_frame_number = time.looping_frame_number(&sprite);
        }
    }

    fn get_sprite_draw_coords(&self, sprite: &SpriteRenderer) -> Vec2 {
        let mut x = self.pos.x;
        let y = self.pos.y;

        if self.is_facing_left {
            x += self.left_facing_x_offset;
        }

        match self.rotation {
            Rotation::None => Vec2::new(x, y),
            // Macroquad rotates sprites around their center, so we're going to
            // "undo" that by translating by the opposite amount. Note that while
            // Macroquad *does* support specifying an alternative pivot point, it's
            // specified in absolute coordinates and extremely confusing, so we're
            // just doing things this way.
            Rotation::Clockwise270 => {
                let half_unrotated_frame_width = sprite.frame_width() / 2.;
                let half_unrotated_frame_height = sprite.frame_height() / 2.;
                Vec2::new(
                    x - half_unrotated_frame_width + half_unrotated_frame_height,
                    y + half_unrotated_frame_width - half_unrotated_frame_height,
                )
            }
        }
    }

    pub fn draw_current_frame(&self, level: &Level) {
        self.material.start_using();
        match self.renderer {
            Renderer::Invisible => {}
            Renderer::Sprite => {
                if let Some(sprite) = self.sprite {
                    let pos = self.get_sprite_draw_coords(sprite);
                    sprite.draw_ex(
                        pos.x,
                        pos.y,
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
                } else {
                    println!("Warning: Renderer::Sprite is used but no sprite is defined!");
                }
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
        if let Some(sprite) = self.sprite {
            let frame = self.get_sprite_dimensions(&sprite);
            draw_rectangle_lines(self.pos.x, self.pos.y, frame.x, frame.y, 1.0, GREEN);
        }
        draw_rect_lines(&self.bbox(), 2., PURPLE);
    }
}
