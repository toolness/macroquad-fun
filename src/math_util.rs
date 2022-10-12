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

/// Scale the rect's position and size. This is different
/// from Rect.scale(), which only scales the size.
pub fn scale_rect_position_and_size(rect: &Rect, amount: f32) -> Rect {
    Rect::new(
        rect.x * amount,
        rect.y * amount,
        rect.w * amount,
        rect.h * amount,
    )
}

/// Shrink the rectangle by the given x and y amounts, using
/// its center as the origin.
pub fn contract_rect_xy(rect: &Rect, x_amount: f32, y_amount: f32) -> Rect {
    let mut result = *rect;

    result.x += x_amount;
    result.y += y_amount;
    result.w -= x_amount * 2.;
    result.h -= y_amount * 2.;

    return result;
}

/// Shrink the rectangle by the given amount along both axes, using
/// its center as the origin.
pub fn contract_rect(rect: &Rect, amount: f32) -> Rect {
    contract_rect_xy(&rect, amount, amount)
}
