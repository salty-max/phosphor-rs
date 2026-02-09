//! The `frame` module provides a high-level drawing API.
//!
//! A [`Frame`] wraps a [`Buffer`] and provides methods
//! for drawing text, shapes, and widgets without having to manipulate
//! individual cells manually.

use crate::{Buffer, Style};

/// A high-level handle for drawing to a buffer.
pub struct Frame<'a> {
    buffer: &'a mut Buffer,
    current_style: Style,
}

impl<'a> Frame<'a> {
    /// Creates a new frame wrapping the given buffer.
    pub fn new(buffer: &'a mut Buffer) -> Self {
        Self {
            buffer,
            current_style: Style::default(),
        }
    }

    /// Returns the width of the frame.
    pub fn width(&self) -> u16 {
        self.buffer.width
    }

    /// Returns the height of the frame.
    pub fn height(&self) -> u16 {
        self.buffer.height
    }

    /// Writes a string to the buffer starting at the given coordinates.
    ///
    /// Text that exceeds the buffer width will be clipped.
    pub fn write_str(&mut self, x: u16, y: u16, text: &str) {
        for (i, c) in text.chars().enumerate() {
            self.buffer
                .set_with_style(x + (i as u16), y, c, self.current_style);
        }
    }

    /// Sets the style to be used for all subsequent drawing operations.
    pub fn set_style(&mut self, style: Style) {
        self.current_style = style
    }

    /// Resets the current style to the default (no colors, no modifiers).
    pub fn reset_style(&mut self) {
        self.current_style = Style::default()
    }

    /// Executes a closure with a specific style, then restores the previous style.
    ///
    /// This is useful for drawing a specific section of the UI with a different
    /// style without affecting subsequent drawing operations.
    pub fn with_style<F>(&mut self, style: Style, f: F)
    where
        F: FnOnce(&mut Frame),
    {
        let old_style = self.current_style;
        self.current_style = style;
        f(self);
        self.current_style = old_style;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Buffer;
    use crate::Color;

    #[test]
    fn test_frame_with_style_scoped() {
        let mut buffer = Buffer::new(10, 1);
        let mut frame = Frame::new(&mut buffer);
        let red = Style::new().fg(Color::Red);
        let blue = Style::new().fg(Color::Blue);

        frame.set_style(blue);
        frame.with_style(red, |f| {
            f.write_str(0, 0, "R");
            assert_eq!(f.current_style.foreground, Some(Color::Red));
        });

        frame.write_str(1, 0, "B");

        assert_eq!(buffer.get(0, 0).symbol, 'R');
        assert_eq!(buffer.get(0, 0).style.foreground, Some(Color::Red));
        assert_eq!(buffer.get(1, 0).symbol, 'B');
        assert_eq!(buffer.get(1, 0).style.foreground, Some(Color::Blue));
    }

    #[test]
    fn test_frame_styled_write_str() {
        let mut buffer = Buffer::new(10, 1);
        let mut frame = Frame::new(&mut buffer);
        let style = Style::new().fg(Color::Red);

        frame.set_style(style);
        frame.write_str(0, 0, "A");

        assert_eq!(buffer.get(0, 0).symbol, 'A');
        assert_eq!(buffer.get(0, 0).style.foreground, Some(Color::Red));
    }

    #[test]
    fn test_frame_write_str() {
        let mut buffer = Buffer::new(10, 1);
        let mut frame = Frame::new(&mut buffer);

        frame.write_str(2, 0, "Hello");

        assert_eq!(buffer.get(1, 0).symbol, ' ');
        assert_eq!(buffer.get(2, 0).symbol, 'H');
        assert_eq!(buffer.get(6, 0).symbol, 'o');
        assert_eq!(buffer.get(7, 0).symbol, ' ');
    }

    #[test]
    fn test_frame_write_str_clipping() {
        let mut buffer = Buffer::new(5, 1);
        let mut frame = Frame::new(&mut buffer);

        // "Hello World" is 11 chars, buffer is 5.
        // Starting at 2, it should only write "Hel"
        frame.write_str(2, 0, "Hello World");

        assert_eq!(buffer.get(1, 0).symbol, ' ');
        assert_eq!(buffer.get(2, 0).symbol, 'H');
        assert_eq!(buffer.get(4, 0).symbol, 'l');
    }
}
