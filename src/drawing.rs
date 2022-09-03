use macroquad::prelude::*;

pub fn draw_rect_lines(collider: &Rect, thickness: f32, color: Color) {
    draw_rectangle_lines(
        collider.left(),
        collider.top(),
        collider.size().x,
        collider.size().y,
        thickness,
        color,
    );
}
