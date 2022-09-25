use macroquad::prelude::*;

pub fn draw_rect_lines(rect: &Rect, thickness: f32, color: Color) {
    draw_rectangle_lines(
        rect.left(),
        rect.top(),
        rect.size().x,
        rect.size().y,
        thickness,
        color,
    );
}
