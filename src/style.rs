//! The `style` module provides types for customizing the appearance of text.
//!
//! It supports ANSI colors and text modifiers like Bold, Italic, and Underline.

/// Represents a color in the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

impl Color {
    pub fn from_hex(hex: &str) -> Option<Self> {
        let s = hex.strip_prefix("#").unwrap_or(hex);
        if s.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;

        Some(Color::Rgb(r, g, b))
    }

    pub fn to_ansi_fg(&self) -> String {
        match self {
            Color::Reset => "39".to_string(),
            Color::Black => "30".to_string(),
            Color::Red => "31".to_string(),
            Color::Green => "32".to_string(),
            Color::Yellow => "33".to_string(),
            Color::Blue => "34".to_string(),
            Color::Magenta => "35".to_string(),
            Color::Cyan => "36".to_string(),
            Color::White => "37".to_string(),
            Color::BrightBlack => "90".to_string(),
            Color::BrightRed => "91".to_string(),
            Color::BrightGreen => "92".to_string(),
            Color::BrightYellow => "93".to_string(),
            Color::BrightBlue => "94".to_string(),
            Color::BrightMagenta => "95".to_string(),
            Color::BrightCyan => "96".to_string(),
            Color::BrightWhite => "97".to_string(),
            Color::Indexed(i) => format!("38;5;{}", i),
            Color::Rgb(r, g, b) => format!("38;2;{};{};{}", r, g, b),
        }
    }

    pub fn to_ansi_bg(&self) -> String {
        match self {
            Color::Reset => "49".to_string(),
            Color::Black => "40".to_string(),
            Color::Red => "41".to_string(),
            Color::Green => "42".to_string(),
            Color::Yellow => "43".to_string(),
            Color::Blue => "44".to_string(),
            Color::Magenta => "45".to_string(),
            Color::Cyan => "46".to_string(),
            Color::White => "47".to_string(),
            Color::BrightBlack => "100".to_string(),
            Color::BrightRed => "101".to_string(),
            Color::BrightGreen => "102".to_string(),
            Color::BrightYellow => "103".to_string(),
            Color::BrightBlue => "104".to_string(),
            Color::BrightMagenta => "105".to_string(),
            Color::BrightCyan => "106".to_string(),
            Color::BrightWhite => "107".to_string(),
            Color::Indexed(i) => format!("48;5;{}", i),
            Color::Rgb(r, g, b) => format!("48;2;{};{};{}", r, g, b),
        }
    }
}

/// A bitflag representing text modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifier(u16);

impl Modifier {
    pub const BOLD: Self = Self(0b0000_0001);
    pub const ITALIC: Self = Self(0b0000_0010);
    pub const UNDERLINE: Self = Self(0b0000_0100);
    pub const REVERSED: Self = Self(0b0000_1000);
    pub const DIM: Self = Self(0b0001_0000);

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

impl std::ops::BitOr for Modifier {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Represents the visual style of a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub modifiers: Modifier,
}

impl Style {
    /// Creates a new, default style (no colors, no modifiers).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the foreground color.
    pub fn fg(mut self, color: Color) -> Self {
        self.foreground = Some(color);
        self
    }

    /// Sets the background color.
    pub fn bg(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Adds a modifier.
    pub fn modifier(mut self, modifier: Modifier) -> Self {
        self.modifiers.insert(modifier);
        self
    }

    pub fn to_ansi(&self) -> String {
        let mut codes = vec!["0".to_string()];

        if let Some(fg) = self.foreground {
            codes.push(fg.to_ansi_fg());
        }
        if let Some(bg) = self.background {
            codes.push(bg.to_ansi_bg());
        }
        if self.modifiers.contains(Modifier::BOLD) {
            codes.push("1".to_string());
        }
        if self.modifiers.contains(Modifier::DIM) {
            codes.push("2".to_string());
        }
        if self.modifiers.contains(Modifier::ITALIC) {
            codes.push("3".to_string());
        }
        if self.modifiers.contains(Modifier::UNDERLINE) {
            codes.push("4".to_string());
        }
        if self.modifiers.contains(Modifier::REVERSED) {
            codes.push("7".to_string());
        }

        format!("\x1b[{}m", codes.join(";"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_builder() {
        let style = Style::new()
            .fg(Color::Red)
            .bg(Color::Blue)
            .modifier(Modifier::BOLD | Modifier::ITALIC);

        assert_eq!(style.foreground, Some(Color::Red));
        assert_eq!(style.background, Some(Color::Blue));
        assert!(style.modifiers.contains(Modifier::BOLD));
        assert!(style.modifiers.contains(Modifier::ITALIC));
        assert!(!style.modifiers.contains(Modifier::UNDERLINE));
    }

    #[test]
    fn test_color_from_hex() {
        assert_eq!(Color::from_hex("#FF5733"), Some(Color::Rgb(255, 87, 51)));
        assert_eq!(Color::from_hex("000000"), Some(Color::Rgb(0, 0, 0)));
        assert_eq!(Color::from_hex("FFFFFF"), Some(Color::Rgb(255, 255, 255)));
        assert_eq!(Color::from_hex("#123"), None);
        assert_eq!(Color::from_hex("invalid"), None);
    }

    #[test]
    fn test_color_to_ansi() {
        assert_eq!(Color::Red.to_ansi_fg(), "31");
        assert_eq!(Color::BrightBlue.to_ansi_bg(), "104");
        assert_eq!(Color::Rgb(10, 20, 30).to_ansi_fg(), "38;2;10;20;30");
        assert_eq!(Color::Indexed(123).to_ansi_bg(), "48;5;123");
    }

    #[test]
    fn test_style_to_ansi() {
        // Default style (just Reset)
        assert_eq!(Style::default().to_ansi(), "\x1b[0m");

        // Complex style
        let style = Style::new()
            .fg(Color::Red)
            .bg(Color::Blue)
            .modifier(Modifier::BOLD);

        // Note: The order depends on your implementation.
        // Assuming: Reset; FG; BG; Modifiers
        assert_eq!(style.to_ansi(), "\x1b[0;31;44;1m");
    }
}
