//! A container widget with optional borders and title.

use crate::{Frame, Rect, Style, widgets::Widget};

const P_BORDER_H: char = '\u{2500}';
const P_BORDER_V: char = '\u{2502}';
const P_BORDER_TL: char = '\u{250C}';
const P_BORDER_TR: char = '\u{2510}';
const P_BORDER_BL: char = '\u{2514}';
const P_BORDER_BR: char = '\u{2518}';

const R_BORDER_H: char = '\u{2500}';
const R_BORDER_V: char = '\u{2502}';
const R_BORDER_TL: char = '\u{256D}';
const R_BORDER_TR: char = '\u{256E}';
const R_BORDER_BL: char = '\u{2570}';
const R_BORDER_BR: char = '\u{256F}';

const D_BORDER_H: char = '\u{2550}';
const D_BORDER_V: char = '\u{2551}';
const D_BORDER_TL: char = '\u{2554}';
const D_BORDER_TR: char = '\u{2557}';
const D_BORDER_BL: char = '\u{255A}';
const D_BORDER_BR: char = '\u{255D}';

/// The style of the borders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderType {
    /// Standard thin lines (┌, ─, ┐, etc.).
    Plain,
    /// Rounded corners (╭, ─, ╮, etc.).
    #[default]
    Rounded,
    /// Double lines (╔, ═, ╗, etc.).
    Double,
}

impl BorderType {
    /// Returns the characters used to draw the borders in the following order:
    /// (Horizontal, Vertical, Top-Left, Top-Right, Bottom-Left, Bottom-Right)
    pub fn get_chars(&self) -> (char, char, char, char, char, char) {
        match self {
            BorderType::Plain => (
                P_BORDER_H,
                P_BORDER_V,
                P_BORDER_TL,
                P_BORDER_TR,
                P_BORDER_BL,
                P_BORDER_BR,
            ),
            BorderType::Rounded => (
                R_BORDER_H,
                R_BORDER_V,
                R_BORDER_TL,
                R_BORDER_TR,
                R_BORDER_BL,
                R_BORDER_BR,
            ),
            BorderType::Double => (
                D_BORDER_H,
                D_BORDER_V,
                D_BORDER_TL,
                D_BORDER_TR,
                D_BORDER_BL,
                D_BORDER_BR,
            ),
        }
    }
}

/// Bitflag representing which sides of a block should have borders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Borders(u8);

impl Borders {
    pub const NONE: Self = Self(0b0000);
    pub const TOP: Self = Self(0b0001);
    pub const RIGHT: Self = Self(0b0010);
    pub const BOTTOM: Self = Self(0b0100);
    pub const LEFT: Self = Self(0b1000);
    pub const ALL: Self = Self(0b1111);

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for Borders {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// A container widget that can display a border and a title.
pub struct Block {
    title: Option<String>,
    borders: Borders,
    border_type: BorderType,
    style: Style,
    title_style: Style,
    padding_x: u16,
    padding_y: u16,
}

impl Default for Block {
    fn default() -> Self {
        Self::new()
    }
}

impl Block {
    pub fn new() -> Self {
        Self {
            title: None,
            borders: Borders::NONE,
            border_type: BorderType::Rounded,
            style: Style::default(),
            title_style: Style::default(),
            padding_x: 0,
            padding_y: 0,
        }
    }

    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn borders(mut self, borders: Borders) -> Self {
        self.borders = borders;
        self
    }

    pub fn border_type(mut self, border_type: BorderType) -> Self {
        self.border_type = border_type;
        self
    }

    /// Sets the style of the borders.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the style of the title.
    pub fn title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    /// Sets both horizontal and vertical padding to the same value.
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding_x = padding;
        self.padding_y = padding;
        self
    }

    /// Sets the horizontal padding.
    pub fn padding_x(mut self, padding: u16) -> Self {
        self.padding_x = padding;
        self
    }

    /// Sets the vertical padding.
    pub fn padding_y(mut self, padding: u16) -> Self {
        self.padding_y = padding;
        self
    }

    /// Returns the inner area of the block, excluding the borders and padding.
    ///
    /// This is useful for rendering other widgets inside the block.
    pub fn inner(&self, area: Rect) -> Rect {
        let mut x = area.x;
        let mut y = area.y;
        let mut w = area.width;
        let mut h = area.height;

        if self.borders.contains(Borders::LEFT) {
            x += 1;
            w = w.saturating_sub(1);
        }
        if self.borders.contains(Borders::TOP) {
            y += 1;
            h = h.saturating_sub(1);
        }
        if self.borders.contains(Borders::RIGHT) {
            w = w.saturating_sub(1);
        }
        if self.borders.contains(Borders::BOTTOM) {
            h = h.saturating_sub(1);
        }

        x += self.padding_x;
        y += self.padding_y;
        w = w.saturating_sub(self.padding_x * 2);
        h = h.saturating_sub(self.padding_y * 2);

        Rect::new(x, y, w, h)
    }
}

impl Widget for Block {
    fn render(self, area: Rect, frame: &mut Frame) {
        let (h, v, tl, tr, bl, br) = self.border_type.get_chars();
        frame.with_style(self.style, |f| {
            f.render_area(area, |f| {
                let width = f.width();
                let height = f.height();
                let mut buf = [0u8; 4];

                // 1. Draw Sides (Edge-to-Edge)
                if self.borders.contains(Borders::TOP) {
                    let s = h.encode_utf8(&mut buf);
                    for x in 0..width {
                        f.write_str(x, 0, s);
                    }
                }
                if self.borders.contains(Borders::BOTTOM) {
                    let s = h.encode_utf8(&mut buf);
                    for x in 0..width {
                        f.write_str(x, height - 1, s);
                    }
                }
                if self.borders.contains(Borders::LEFT) {
                    let s = v.encode_utf8(&mut buf);
                    for y in 0..height {
                        f.write_str(0, y, s);
                    }
                }
                if self.borders.contains(Borders::RIGHT) {
                    let s = v.encode_utf8(&mut buf);
                    for y in 0..height {
                        f.write_str(width - 1, y, s);
                    }
                }

                // 2. Draw Corners (Overwrite intersections)
                let corners = [
                    (0, 0, Borders::TOP | Borders::LEFT, tl),
                    (width - 1, 0, Borders::TOP | Borders::RIGHT, tr),
                    (0, height - 1, Borders::BOTTOM | Borders::LEFT, bl),
                    (width - 1, height - 1, Borders::BOTTOM | Borders::RIGHT, br),
                ];

                for (x, y, req, sym) in corners {
                    if self.borders.contains(req) {
                        f.write_str(x, y, sym.encode_utf8(&mut buf));
                    }
                }

                // 3. Draw Title
                if let Some(t) = self.title {
                    let style = if self.title_style == Style::default() {
                        self.style
                    } else {
                        self.title_style
                    };

                    f.with_style(style, |f| {
                        f.write_str(2, 0, &format!(" {} ", t));
                    });
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Buffer;

    #[test]
    fn test_block_render_borders() {
        let mut buffer = Buffer::new(5, 3);
        let mut frame = Frame::new(&mut buffer, Rect::new(0, 0, 5, 3));
        let block = Block::new().borders(Borders::ALL);

        block.render(Rect::new(0, 0, 5, 3), &mut frame);

        // Corners
        assert_eq!(buffer.get(0, 0).symbol, R_BORDER_TL);
        assert_eq!(buffer.get(4, 0).symbol, R_BORDER_TR);
        assert_eq!(buffer.get(0, 2).symbol, R_BORDER_BL);
        assert_eq!(buffer.get(4, 2).symbol, R_BORDER_BR);

        // Sides
        assert_eq!(buffer.get(2, 0).symbol, R_BORDER_H);
        assert_eq!(buffer.get(0, 1).symbol, R_BORDER_V);
    }

    #[test]
    fn test_block_render_double_borders() {
        let mut buffer = Buffer::new(5, 3);
        let mut frame = Frame::new(&mut buffer, Rect::new(0, 0, 5, 3));
        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Double);

        block.render(Rect::new(0, 0, 5, 3), &mut frame);

        assert_eq!(buffer.get(0, 0).symbol, D_BORDER_TL);
        assert_eq!(buffer.get(2, 0).symbol, D_BORDER_H);
    }

    #[test]
    fn test_block_render_title() {
        let mut buffer = Buffer::new(10, 3);
        let mut frame = Frame::new(&mut buffer, Rect::new(0, 0, 10, 3));
        let block = Block::new().borders(Borders::TOP).title("Hi");

        block.render(Rect::new(0, 0, 10, 3), &mut frame);

        // Title should be at x=2, y=0, wrapped in spaces
        assert_eq!(buffer.get(2, 0).symbol, ' ');
        assert_eq!(buffer.get(3, 0).symbol, 'H');
        assert_eq!(buffer.get(4, 0).symbol, 'i');
        assert_eq!(buffer.get(5, 0).symbol, ' ');
    }

    #[test]
    fn test_block_render_styled_title() {
        use crate::Color;

        let mut buffer = Buffer::new(10, 3);
        let mut frame = Frame::new(&mut buffer, Rect::new(0, 0, 10, 3));
        let block = Block::new()
            .borders(Borders::TOP)
            .title("Hi")
            .title_style(Style::new().fg(Color::Red));

        block.render(Rect::new(0, 0, 10, 3), &mut frame);

        assert_eq!(buffer.get(3, 0).symbol, 'H');
        assert_eq!(buffer.get(3, 0).style.foreground, Some(Color::Red));
    }

    #[test]
    fn test_block_inner_area() {
        let block = Block::new().borders(Borders::ALL).padding(0);
        let area = Rect::new(10, 10, 10, 10);
        let inner = block.inner(area);

        assert_eq!(inner.x, 11);
        assert_eq!(inner.y, 11);
        assert_eq!(inner.width, 8);
        assert_eq!(inner.height, 8);
    }

    #[test]
    fn test_block_padding() {
        let block = Block::new().borders(Borders::NONE).padding(2);
        let area = Rect::new(0, 0, 10, 10);
        let inner = block.inner(area);

        assert_eq!(inner.x, 2);
        assert_eq!(inner.y, 2);
        assert_eq!(inner.width, 6);
        assert_eq!(inner.height, 6);
    }

    #[test]
    fn test_block_granular_padding() {
        let block = Block::new()
            .borders(Borders::NONE)
            .padding_x(2)
            .padding_y(1);
        let area = Rect::new(0, 0, 10, 10);
        let inner = block.inner(area);

        assert_eq!(inner.x, 2);
        assert_eq!(inner.y, 1);
        assert_eq!(inner.width, 6); // 10 - (2 * 2)
        assert_eq!(inner.height, 8); // 10 - (1 * 2)
    }
}
