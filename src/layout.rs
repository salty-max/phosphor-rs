//! The `layout` module provides tools for dividing terminal space.
//!
//! The core type is [`Rect`], which represents a rectangular area on the screen.
//! The [`Layout`] engine can split a [`Rect`] into multiple sub-rectangles based on [`Constraint`]s.

/// The direction in which a rectangle is split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Split horizontally (side-by-side).
    Horizontal,
    /// Split vertically (top-to-bottom).
    Vertical,
}

/// Constraints used to define the size of a layout segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constraint {
    /// Takes up the remaining available space.
    ///
    /// If multiple `Fill` constraints are used, the remaining space is divided
    /// equally among them.
    Fill,
    /// A fixed percentage of the available space (0-100).
    Percentage(u16),
    /// A fixed number of cells.
    Length(u16),
    /// A ratio of the available space (e.g., `Ratio(1, 3)` for one third).
    Ratio(u32, u32),
    /// Takes up `Fill` space, but is at least `u16` cells.
    Min(u16),
    /// Takes up `Fill` space, but is at most `u16` cells.
    Max(u16),
}

/// A rectangular area on the screen.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// The horizontal coordinate of the top-left corner.
    pub x: u16,
    /// The vertical coordinate of the top-left corner.
    pub y: u16,
    /// The width of the rectangle in columns.
    pub width: u16,
    /// The height of the rectangle in rows.
    pub height: u16,
}

impl Rect {
    /// Creates a new rectangle.
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the total number of cells in the rectangle.
    pub fn area(&self) -> u16 {
        self.width * self.height
    }

    /// Returns the x-coordinate of the left edge.
    pub fn left(&self) -> u16 {
        self.x
    }

    /// Returns the x-coordinate of the right edge.
    pub fn right(&self) -> u16 {
        self.x + self.width
    }

    /// Returns the y-coordinate of the top edge.
    pub fn top(&self) -> u16 {
        self.y
    }

    /// Returns the y-coordinate of the bottom edge.
    pub fn bottom(&self) -> u16 {
        self.y + self.height
    }
}

/// A layout engine that divides a rectangle into sub-rectangles based on constraints.
pub struct Layout {
    /// The direction of the split.
    pub direction: Direction,
    /// The constraints for each segment.
    pub constraints: Vec<Constraint>,
}

impl Layout {
    /// Creates a new layout.
    pub fn new(direction: Direction, constraints: Vec<Constraint>) -> Self {
        Self {
            direction,
            constraints,
        }
    }

    /// Splits the given rectangle into sub-rectangles.
    ///
    /// The number of returned rectangles matches the number of constraints.
    pub fn split(&self, rect: Rect) -> Vec<Rect> {
        let mut rects = Vec::new();
        let total_space = match &self.direction {
            Direction::Horizontal => rect.width,
            Direction::Vertical => rect.height,
        };

        let start_x = rect.x;
        let start_y = rect.y;
        let mut offset = 0;

        // 1. Calculate used space and count fills
        let mut used_space = 0;
        let mut flex_count = 0;

        for c in &self.constraints {
            match c {
                Constraint::Length(l) => used_space += l,
                Constraint::Percentage(p) => used_space += (p * total_space) / 100,
                Constraint::Ratio(n, d) => used_space += (total_space as u32 * n / d) as u16,
                Constraint::Fill | Constraint::Min(_) | Constraint::Max(_) => flex_count += 1,
            }
        }

        // 2. Calculate size of one `Fill` unit
        let flex_size = if flex_count > 0 {
            total_space.saturating_sub(used_space) / flex_count
        } else {
            0
        };

        // 3. Create rects
        for c in &self.constraints {
            let size = match c {
                Constraint::Length(l) => *l,
                Constraint::Percentage(p) => (p * total_space) / 100,
                Constraint::Fill => flex_size,
                Constraint::Ratio(n, d) => (total_space as u32 * n / d) as u16,
                Constraint::Min(n) => flex_size.max(*n),
                Constraint::Max(n) => flex_size.min(*n),
            };

            let sub_rect = match &self.direction {
                Direction::Horizontal => Rect::new(start_x + offset, start_y, size, rect.height),
                Direction::Vertical => Rect::new(start_x, start_y + offset, rect.width, size),
            };

            rects.push(sub_rect);
            offset += size;
        }

        rects
    }

    /// Splits the given rectangle into a fixed-size array of sub-rectangles.
    ///
    /// # Panics
    /// Panics if the number of constraints does not match the array size `N`.
    pub fn split_to<const N: usize>(&self, rect: Rect) -> [Rect; N] {
        let rects = self.split(rect);
        rects.try_into().expect("Layout constraints count mismatch")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_calculations() {
        let rect = Rect::new(10, 10, 20, 5);
        assert_eq!(rect.area(), 100);
        assert_eq!(rect.left(), 10);
        assert_eq!(rect.right(), 30);
        assert_eq!(rect.top(), 10);
        assert_eq!(rect.bottom(), 15);
    }

    #[test]
    fn test_layout_split_vertical() {
        let layout = Layout::new(
            Direction::Vertical,
            vec![Constraint::Length(2), Constraint::Percentage(50)],
        );
        let rect = Rect::new(0, 0, 10, 10);
        let rects = layout.split(rect);

        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0], Rect::new(0, 0, 10, 2));
        assert_eq!(rects[1], Rect::new(0, 2, 10, 5));
    }

    #[test]
    fn test_layout_split_fill() {
        let layout = Layout::new(
            Direction::Vertical,
            vec![Constraint::Length(2), Constraint::Fill, Constraint::Fill],
        );
        let rect = Rect::new(0, 0, 10, 10);
        let rects = layout.split(rect);

        assert_eq!(rects.len(), 3);
        assert_eq!(rects[0].height, 2);
        assert_eq!(rects[1].height, 4); // (10 - 2) / 2
        assert_eq!(rects[2].height, 4);
        assert_eq!(rects[2].y, 6);
    }

    #[test]
    fn test_layout_split_to() {
        let layout = Layout::new(
            Direction::Horizontal,
            vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        );
        let rect = Rect::new(0, 0, 100, 10);
        let [left, right] = layout.split_to(rect);

        assert_eq!(left.width, 50);
        assert_eq!(right.width, 50);
        assert_eq!(right.x, 50);
    }

    #[test]
    #[should_panic]
    fn test_layout_split_to_mismatch() {
        let layout = Layout::new(Direction::Vertical, vec![Constraint::Fill]);
        let rect = Rect::new(0, 0, 10, 10);
        let _: [Rect; 2] = layout.split_to(rect); // Should panic
    }

    #[test]
    fn test_layout_split_ratio() {
        let layout = Layout::new(
            Direction::Vertical,
            vec![Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)],
        );
        let rect = Rect::new(0, 0, 100, 100);
        let rects = layout.split(rect);

        assert_eq!(rects[0].height, 25);
        assert_eq!(rects[1].height, 75);
    }

    #[test]
    fn test_layout_split_min_max() {
        let rect = Rect::new(0, 0, 100, 100);

        // Min test: flex_size is 50, but Min is 60
        let layout_min = Layout::new(
            Direction::Vertical,
            vec![Constraint::Fill, Constraint::Min(60)],
        );
        let rects_min = layout_min.split(rect);
        assert_eq!(rects_min[1].height, 60);

        // Max test: flex_size is 50, but Max is 40
        let layout_max = Layout::new(
            Direction::Vertical,
            vec![Constraint::Fill, Constraint::Max(40)],
        );
        let rects_max = layout_max.split(rect);
        assert_eq!(rects_max[1].height, 40);
    }
}
