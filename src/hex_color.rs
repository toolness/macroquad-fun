use macroquad::prelude::{Color, Vec4};

const fn hex_to_u8(value: char) -> Option<u8> {
    match value {
        '0' => Some(0),
        '1' => Some(1),
        '2' => Some(2),
        '3' => Some(3),
        '4' => Some(4),
        '5' => Some(5),
        '6' => Some(6),
        '7' => Some(7),
        '8' => Some(8),
        '9' => Some(9),
        'a' | 'A' => Some(10),
        'b' | 'B' => Some(11),
        'c' | 'C' => Some(12),
        'd' | 'D' => Some(13),
        'e' | 'E' => Some(14),
        'f' | 'F' => Some(15),
        _ => None,
    }
}

const fn hex_duo_to_u8(first: char, second: char) -> Option<u8> {
    let Some(first_u8) = hex_to_u8(first) else {
        return None;
    };

    let Some(second_u8) = hex_to_u8(second) else {
        return None;
    };

    Some((first_u8 << 4) + second_u8)
}

const fn hex_color_to_opt_u8_slice(value: &'static str) -> Option<[u8; 3]> {
    let bytes = value.as_bytes();

    if bytes.len() == 0 {
        return None;
    }

    let start = if bytes[0] as char == '#' { 1 } else { 0 };

    if bytes.len() < start + 6 {
        return None;
    }

    let Some(r) = hex_duo_to_u8(bytes[start + 0] as char, bytes[start + 1] as char) else {
        return None;
    };

    let Some(g) = hex_duo_to_u8(bytes[start + 2] as char, bytes[start + 3] as char) else {
        return None;
    };

    let Some(b) = hex_duo_to_u8(bytes[start + 4] as char, bytes[start + 5] as char) else {
        return None;
    };

    Some([r, g, b])
}

// It'd be nice to also convert this to floating point, but we can't yet:
// https://github.com/rust-lang/rust/issues/57241
pub const fn hex_color(value: &'static str) -> HexColor {
    let Some(color) = hex_color_to_opt_u8_slice(value) else {
        panic!("Unable to parse hex color!");
    };

    HexColor { color }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct HexColor {
    color: [u8; 3],
}

impl HexColor {
    pub fn vec4(&self) -> Vec4 {
        Vec4::new(
            self.color[0] as f32 / 255.,
            self.color[1] as f32 / 255.,
            self.color[2] as f32 / 255.,
            1.0,
        )
    }
}

impl Into<Color> for HexColor {
    fn into(self) -> Color {
        Color::new(
            self.color[0] as f32 / 255.,
            self.color[1] as f32 / 255.,
            self.color[2] as f32 / 255.,
            1.,
        )
    }
}

#[cfg(test)]
mod tests {
    use macroquad::prelude::{Color, BLACK};

    use crate::hex_color::{hex_color, HexColor};

    use super::hex_color_to_opt_u8_slice;

    #[test]
    fn test_short_strings_return_none() {
        assert_eq!(hex_color_to_opt_u8_slice(""), None);
        assert_eq!(hex_color_to_opt_u8_slice("#"), None);
        assert_eq!(hex_color_to_opt_u8_slice("#abcde"), None);
    }

    #[test]
    fn test_invalid_hex_codes_returns_none() {
        assert_eq!(hex_color_to_opt_u8_slice("zzzzzz"), None);
        assert_eq!(hex_color_to_opt_u8_slice("zz0000"), None);
        assert_eq!(hex_color_to_opt_u8_slice("00zz00"), None);
        assert_eq!(hex_color_to_opt_u8_slice("0000zz"), None);
    }

    #[test]
    fn test_without_hash_works() {
        assert_eq!(hex_color_to_opt_u8_slice("000000"), Some([0, 0, 0]));
        assert_eq!(hex_color_to_opt_u8_slice("ff1831"), Some([255, 24, 49]));
        assert_eq!(hex_color_to_opt_u8_slice("ffffff"), Some([255, 255, 255]));
    }

    #[test]
    fn test_with_hash_works() {
        assert_eq!(hex_color_to_opt_u8_slice("#000000"), Some([0, 0, 0]));
        assert_eq!(hex_color_to_opt_u8_slice("#ff1831"), Some([255, 24, 49]));
        assert_eq!(hex_color_to_opt_u8_slice("#ffffff"), Some([255, 255, 255]));
    }

    #[test]
    fn test_hex_color_works() {
        const BLACK_FROM_HEX: HexColor = hex_color("#000000");
        assert_eq!(BLACK_FROM_HEX, HexColor { color: [0, 0, 0] });
        assert_eq!(<HexColor as Into<Color>>::into(BLACK_FROM_HEX), BLACK);
    }
}
