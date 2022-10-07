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

pub fn draw_crosshair(point: &Vec2, radius: f32, thickness: f32, color: Color) {
    draw_line(
        point.x - radius,
        point.y,
        point.x + radius,
        point.y,
        thickness,
        color,
    );
    draw_line(
        point.x,
        point.y - radius,
        point.x,
        point.y + radius,
        thickness,
        color,
    );
}
