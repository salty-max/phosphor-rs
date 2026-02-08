use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Key(KeyEvent),
    Resize(u16, u16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    pub fn new(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::empty(),
        }
    }

    pub fn with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Backspace,
    Esc,
    Left,
    Right,
    Up,
    Down,
    Tab,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
    Null,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct KeyModifiers(u8);

impl KeyModifiers {
    pub const SHIFT: Self = Self(0b0000_0001);
    pub const CTRL: Self = Self(0b0000_0010);
    pub const ALT: Self = Self(0b0000_0100);

    pub fn empty() -> Self {
        Self(0)
    }

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

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

pub struct Parser {
    buffer: Vec<u8>,
}

impl Parser {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn parse(&mut self, bytes: &[u8]) -> Vec<Event> {
        self.buffer.extend_from_slice(bytes);
        let mut events: Vec<Event> = Vec::new();

        loop {
            if self.buffer.is_empty() {
                break;
            }

            match self.buffer[0] {
                b'\r' => {
                    events.push(Event::Key(KeyEvent::new(KeyCode::Enter)));
                    self.buffer.remove(0);
                }
                b'\x1b' => {
                    if self.buffer.len() == 1 {
                        break; // Wait for more
                    }

                    if self.buffer.len() >= 3 && self.buffer[1] == b'[' {
                        match self.buffer[2] {
                            b'A' => {
                                events.push(Event::Key(KeyEvent::new(KeyCode::Up)));
                                self.buffer.drain(0..3);
                            }
                            _ => {
                                // Unknown CSI, just consume Esc
                                events.push(Event::Key(KeyEvent::new(KeyCode::Esc)));
                                self.buffer.remove(0);
                            }
                        }
                    } else {
                        // Just Esc
                        events.push(Event::Key(KeyEvent::new(KeyCode::Esc)));
                        self.buffer.remove(0);
                    }
                }
                b => {
                    let width = utf8_char_width(b);
                    if width == 0 {
                        // Invalid, consume 1 byte
                        self.buffer.remove(0);
                    } else if self.buffer.len() >= width {
                        let s = std::str::from_utf8(&self.buffer[0..width]).unwrap();
                        let c = s.chars().next().unwrap();
                        events.push(Event::Key(KeyEvent::new(KeyCode::Char(c))));
                        self.buffer.drain(0..width);
                    } else {
                        // Wait for more data
                        break;
                    }
                }
            }
        }

        events
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
    } // Invalid
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
        // 'é' is 0xC3 0xA9
        let events = parser.parse(&[0xc3, 0xa9]);
        assert_eq!(events, vec![Event::Key(KeyEvent::new(KeyCode::Char('é')))]);
    }
}
