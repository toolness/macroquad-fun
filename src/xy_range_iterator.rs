pub struct XYRangeIterator {
    x_start: i64,
    x_end: i64,
    y_end: i64,
    x: i64,
    y: i64,
}

impl XYRangeIterator {
    pub fn new(x1: i64, y1: i64, x2: i64, y2: i64) -> Self {
        XYRangeIterator {
            x_start: x1,
            x_end: x2,
            y_end: y2,
            x: x1 - 1,
            y: y1,
        }
    }
}

impl Iterator for XYRangeIterator {
    type Item = (i64, i64);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y > self.y_end {
            return None;
        }
        if self.x < self.x_end {
            self.x += 1;
        } else {
            self.x = self.x_start;
            self.y += 1;
            if self.y > self.y_end {
                return None;
            }
        }
        return Some((self.x, self.y));
    }
}

#[cfg(test)]
mod tests {
    use super::XYRangeIterator;

    #[test]
    fn test_it_works_with_unit_square_at_origin() {
        let coords: Vec<(i64, i64)> = XYRangeIterator::new(0, 0, 1, 1).collect();
        assert_eq!(coords, vec![(0, 0), (1, 0), (0, 1), (1, 1)]);
    }

    #[test]
    fn test_it_works_on_non_square_rectangles() {
        let coords: Vec<(i64, i64)> = XYRangeIterator::new(0, 0, 1, 2).collect();
        assert_eq!(coords, vec![(0, 0), (1, 0), (0, 1), (1, 1), (0, 2), (1, 2)]);
    }

    #[test]
    fn test_it_works_when_width_is_zero() {
        let coords: Vec<(i64, i64)> = XYRangeIterator::new(0, 0, 0, 1).collect();
        assert_eq!(coords, vec![(0, 0), (0, 1)]);
    }

    #[test]
    fn test_it_works_when_height_is_zero() {
        let coords: Vec<(i64, i64)> = XYRangeIterator::new(0, 0, 1, 0).collect();
        assert_eq!(coords, vec![(0, 0), (1, 0)]);
    }

    #[test]
    fn test_it_works_away_from_origin() {
        let coords: Vec<(i64, i64)> = XYRangeIterator::new(3, 4, 4, 5).collect();
        assert_eq!(coords, vec![(3, 4), (4, 4), (3, 5), (4, 5)]);
    }
}
