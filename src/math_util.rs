use macroquad::prelude::Rect;

/// Returns if the given floats have opposite
/// positive/negative signs.
pub fn are_opposites(a: f32, b: f32) -> bool {
    a > 0. && b < 0. || a < 0. && b > 0.
}

/// Returns a Rect with the floor of every
/// dimension. Useful to avoid visual artifacting
/// when drawing.
pub fn floor_rect(rect: &Rect) -> Rect {
    Rect::new(
        rect.x.floor(),
        rect.y.floor(),
        rect.w.floor(),
        rect.h.floor(),
    )
}
