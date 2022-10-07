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

/// Shrink the rectangle by the given amount, using
/// its center as the origin.
pub fn contract_rect(rect: &Rect, x_amount: f32, y_amount: f32) -> Rect {
    let mut result = *rect;

    result.x += x_amount;
    result.y += y_amount;
    result.w -= x_amount * 2.;
    result.h -= y_amount * 2.;

    return result;
}
