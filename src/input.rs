//! The `input` module handles the parsing of raw byte streams into semantic events.
//!
//! It provides a robust state machine ([`Parser`]) that can
//! handle fragmented ANSI escape sequences and multi-byte UTF-8 characters.
//!
//! # Architecture
//! * [`Event`]: The high-level representation of user input.
//! * [`Input`]: The primary interface for reading events from a [`Terminal`].
//! * [`Parser`]: The internal logic for decoding bytes.

use std::collections::VecDeque;
use std::fmt;
use std::time::Duration;

use crate::terminal::Terminal;

/// Represents a distinct event occurring in the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A keyboard press (character, control key, or function key).
    Key(KeyEvent),
    /// A mouse event (click, scroll, etc.).
    Mouse(MouseEvent),
    /// A terminal resize event (columns, rows).
    Resize(u16, u16),
}

/// Represents a mouse event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseEvent {
    /// The column (x) where the event occurred (0-based).
    pub x: u16,
    /// The row (y) where the event occurred (0-based).
    pub y: u16,
    /// The type of mouse event (click, scroll, etc.).
    pub kind: MouseKind,
}

impl MouseEvent {
    /// Creates a new mouse event.
    pub fn new(x: u16, y: u16, kind: MouseKind) -> Self {
        Self { x, y, kind }
    }
}

/// The type of mouse action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseKind {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
    Other,
}

/// Represents a specific key press, including modifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    /// The specific key that was pressed.
    pub code: KeyCode,
    /// Any modifiers held down (Shift, Ctrl, Alt).
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    /// Creates a new `KeyEvent` with no modifiers.
    pub fn new(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::empty(),
        }
    }

    /// Creates a new `KeyEvent` with specific modifiers.
    pub fn with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }
}

/// Represents the key identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyCode {
    /// A standard character key (e.g., 'a', '1', '?').
    Char(char),
    /// The Enter key (`\r`).
    Enter,
    /// The Backspace key (`\x7f`).
    Backspace,
    /// The Escape key (`\x1b`).
    Esc,
    /// Arrow keys.
    Left,
    Right,
    Up,
    Down,
    /// The Tab key.
    Tab,
    /// The Delete key.
    Delete,
    /// Navigation keys.
    Home,
    End,
    PageUp,
    PageDown,
    /// Function keys (F1-F12).
    F(u8),
    /// A null byte or empty event.
    Null,
}

/// A bitflag struct representing Shift, Ctrl, and Alt modifiers.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct KeyModifiers(u8);

impl KeyModifiers {
    /// Shift key modifier.
    pub const SHIFT: Self = Self(0b0000_0001);
    /// Control key modifier.
    pub const CTRL: Self = Self(0b0000_0010);
    /// Alt key modifier.
    pub const ALT: Self = Self(0b0000_0100);

    /// Returns an empty set of modifiers.
    pub fn empty() -> Self {
        Self(0)
    }

    /// Checks if a specific modifier is set.
    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Inserts a modifier into the set.
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

impl fmt::Debug for KeyModifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = Vec::new();
        if self.contains(Self::SHIFT) {
            list.push("SHIFT");
        }
        if self.contains(Self::CTRL) {
            list.push("CTRL");
        }
        if self.contains(Self::ALT) {
            list.push("ALT");
        }
        write!(f, "KeyModifiers({:?})", list)
    }
}

impl std::ops::BitOr for KeyModifiers {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Internal state machine for parsing byte streams into Events.
///
/// The parser maintains an internal buffer to handle cases where a single
/// event (like an arrow key) is split across multiple read operations.
pub struct Parser {
    buffer: VecDeque<u8>,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    /// Creates a new, empty parser.
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
        }
    }

    /// Parses a slice of bytes and appends them to the internal buffer,
    /// returning any complete events found.
    ///
    /// This method will consume as many bytes as possible from the buffer
    /// to form complete [`Event`]s.
    pub fn parse(&mut self, bytes: &[u8]) -> Vec<Event> {
        self.buffer.extend(bytes);
        let mut events: Vec<Event> = Vec::new();

        loop {
            if self.buffer.is_empty() {
                break;
            }

            match self.buffer[0] {
                b'\r' => {
                    events.push(Event::Key(KeyEvent::new(KeyCode::Enter)));
                    self.buffer.pop_front();
                }
                b'\x1b' => {
                    if self.buffer.len() == 1 {
                        break; // Incomplete, wait for more data
                    }

                    if self.buffer[1] == b'[' && self.buffer.len() < 3 {
                        break; // Incomplete CSI, wait for more data
                    }

                    if self.buffer.len() >= 3 && self.buffer[1] == b'[' {
                        match self.buffer[2] {
                            b'A' => {
                                events.push(Event::Key(KeyEvent::new(KeyCode::Up)));
                                self.consume(3);
                            }
                            b'M' => {
                                if self.buffer.len() < 6 {
                                    break;
                                }
                                self.consume(3);

                                let cb = self.buffer.pop_front().unwrap();
                                let cx = self.buffer.pop_front().unwrap();
                                let cy = self.buffer.pop_front().unwrap();

                                let kind = match cb.saturating_sub(32) {
                                    0 => MouseKind::LeftClick,
                                    1 => MouseKind::MiddleClick,
                                    2 => MouseKind::RightClick,
                                    64 => MouseKind::ScrollUp,
                                    65 => MouseKind::ScrollDown,
                                    _ => MouseKind::Other,
                                };

                                events.push(Event::Mouse(MouseEvent::new(
                                    (cx.saturating_sub(33)) as u16,
                                    (cy.saturating_sub(33)) as u16,
                                    kind,
                                )));
                            }
                            _ => {
                                events.push(Event::Key(KeyEvent::new(KeyCode::Esc)));
                                self.buffer.pop_front();
                            }
                        }
                    } else {
                        events.push(Event::Key(KeyEvent::new(KeyCode::Esc)));
                        self.buffer.pop_front();
                    }
                }
                b => {
                    let width = utf8_char_width(b);

                    if width == 0 {
                        self.buffer.pop_front();
                    } else if self.buffer.len() >= width {
                        let bytes: Vec<u8> = self.buffer.range(0..width).copied().collect();
                        if let Ok(s) = std::str::from_utf8(&bytes)
                            && let Some(c) = s.chars().next()
                        {
                            events.push(Event::Key(KeyEvent::new(KeyCode::Char(c))));
                        }
                        self.consume(width);
                    } else {
                        break;
                    }
                }
            }
        }

        events
    }

    /// Checks if the parser is holding incomplete data.
    pub fn has_pending_state(&self) -> bool {
        !self.buffer.is_empty()
    }

    /// Forces the parser to interpret whatever is left in the buffer.
    ///
    /// This is called when a timeout occurs during polling, indicating that
    /// an ambiguous sequence (like a lone `\x1b`) should be treated as a
    /// complete event (the `Esc` key).
    pub fn finish_incomplete(&mut self) -> Vec<Event> {
        let mut events: Vec<Event> = Vec::new();
        if self.buffer.is_empty() {
            return events;
        }

        if self.buffer[0] == b'\x1b' {
            events.push(Event::Key(KeyEvent::new(KeyCode::Esc)));
            self.buffer.pop_front();
        }

        self.buffer.clear();
        events
    }

    fn consume(&mut self, n: usize) {
        for _ in 0..n {
            self.buffer.pop_front();
        }
    }
}

fn utf8_char_width(first_byte: u8) -> usize {
    if first_byte & 0b10000000 == 0 {
        1
    } else if first_byte & 0b11100000 == 0b11000000 {
        2
    } else if first_byte & 0b11110000 == 0b11100000 {
        3
    } else if first_byte & 0b11111000 == 0b11110000 {
        4
    } else {
        0
    }
}

/// The main input handler for a Phosphor application.
///
/// It reads raw bytes from the [`Terminal`] and uses an internal [`Parser`]
/// to produce semantic [`Event`]s. It handles the ambiguity of the `Esc` key
/// by polling the terminal for a short duration.
pub struct Input {
    parser: Parser,
}

impl Input {
    /// Creates a new `Input` handler.
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
        }
    }

    /// Reads available bytes from the terminal and returns a vector of parsed events.
    ///
    /// This method will block until at least one byte is read from the terminal.
    /// If the read byte is the start of an escape sequence, it will poll the
    /// terminal for up to 50ms to see if more bytes arrive.
    ///
    /// # Errors
    /// Returns an error if the underlying terminal read or poll fails.
    pub fn read(&mut self, term: &Terminal) -> Vec<Event> {
        let mut buf = [0u8; 1024];
        let mut events: Vec<Event> = Vec::new();

        match term.read(&mut buf) {
            Ok(n) if n > 0 => {
                events.extend(self.parser.parse(&buf[..n]));
            }
            _ => return events,
        }

        while self.parser.has_pending_state() {
            match term.poll(Duration::from_millis(50)) {
                Ok(true) => {
                    if let Ok(n) = term.read(&mut buf) {
                        events.extend(self.parser.parse(&buf[..n]));
                    }
                }
                Ok(false) | Err(_) => {
                    events.extend(self.parser.finish_incomplete());
                    break;
                }
            }
        }

        events
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_char() {
        let mut parser = Parser::new();
        let events = parser.parse(b"a");
        assert_eq!(events, vec![Event::Key(KeyEvent::new(KeyCode::Char('a')))]);
    }

    #[test]
    fn test_parse_enter() {
        let mut parser = Parser::new();
        let events = parser.parse(b"\r");
        assert_eq!(events, vec![Event::Key(KeyEvent::new(KeyCode::Enter))]);
    }

    #[test]
    fn test_parse_arrow() {
        let mut parser = Parser::new();
        let events = parser.parse(b"\x1b[A");
        assert_eq!(events, vec![Event::Key(KeyEvent::new(KeyCode::Up))]);
    }

    #[test]
    fn test_parse_multiple() {
        let mut parser = Parser::new();
        let events = parser.parse(b"a\rb");
        assert_eq!(
            events,
            vec![
                Event::Key(KeyEvent::new(KeyCode::Char('a'))),
                Event::Key(KeyEvent::new(KeyCode::Enter)),
                Event::Key(KeyEvent::new(KeyCode::Char('b'))),
            ]
        );
    }

    #[test]
    fn test_parse_utf8() {
        let mut parser = Parser::new();
        // 'é' is 0xC3 0xA9 in UTF-8
        let events = parser.parse(&[0xc3, 0xa9]);
        assert_eq!(events, vec![Event::Key(KeyEvent::new(KeyCode::Char('é')))]);
    }

    #[test]
    fn test_parse_mouse_click() {
        let mut parser = Parser::new();
        // \x1b[M + (0+32) + (10+33) + (5+33) -> Left click at 10, 5
        // 0+32 = 32 (' ')
        // 10+33 = 43 ('+')
        // 5+33 = 38 ('&')
        let events = parser.parse(b"\x1b[M +&");

        assert_eq!(events.len(), 1);
        if let Event::Mouse(mouse) = &events[0] {
            assert_eq!(mouse.kind, MouseKind::LeftClick);
            assert_eq!(mouse.x, 10);
            assert_eq!(mouse.y, 5);
        } else {
            panic!("Expected Mouse event");
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::terminal::{Terminal, mocks::MockSystem}; // FIX: Reusing the existing MockSystem

    #[test]
    fn test_input_read() {
        // Arrange
        let mock = MockSystem::new();
        // Pre-fill the mock's input buffer with data "a"
        mock.push_input(b"a");

        let term = Terminal::new_with_system(Box::new(mock)).unwrap();
        let mut input = Input::new();

        // Act
        let events = input.read(&term);

        // Assert
        assert_eq!(events, vec![Event::Key(KeyEvent::new(KeyCode::Char('a')))]);
    }

    #[test]
    fn test_input_esc_timeout() {
        // Arrange: Lone Esc byte
        let mock = MockSystem::new();
        mock.push_input(b"\x1b");

        let term = Terminal::new_with_system(Box::new(mock)).unwrap();
        let mut input = Input::new();

        // Act: Read should see Esc, then poll will return false (empty buffer)
        let events = input.read(&term);

        // Assert: Should be interpreted as Esc key
        assert_eq!(events, vec![Event::Key(KeyEvent::new(KeyCode::Esc))]);
    }

    #[test]
    fn test_input_split_arrow() {
        // Arrange: Split Up Arrow sequence (\x1b[A)
        // We use with_max_read(1) to force chunked reads
        let mock = MockSystem::new().with_max_read(1);
        mock.push_input(b"\x1b[A");

        let term = Terminal::new_with_system(Box::new(mock)).unwrap();
        let mut input = Input::new();

        // Act:
        // 1. First read gets \x1b
        // 2. has_pending_state is true
        // 3. poll is called, returns true (buffer has [A)
        // 4. Second read gets [A
        // 5. Parser combines them into Up Arrow
        let events = input.read(&term);

        // Assert
        assert_eq!(events, vec![Event::Key(KeyEvent::new(KeyCode::Up))]);
    }
}
