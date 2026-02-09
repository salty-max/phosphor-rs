//! The `renderer` module handles the actual drawing to the terminal.
//!
//! It uses a [`Buffer`] to track the current state of the
//! screen and only sends the minimal set of ANSI escape codes to update it.

use crate::buffer::Buffer;
use crate::terminal::Terminal;
use std::io;

/// The primary rendering engine.
pub struct Renderer {
    /// The state of the terminal as of the last render.
    current_buffer: Buffer,
}

impl Renderer {
    /// Creates a new renderer for a terminal of the given size.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            current_buffer: Buffer::new(width, height),
        }
    }

    /// Updates the terminal to match the state of the given buffer.
    ///
    /// This method calculates the difference between the new buffer and the
    /// previous one, and only writes the changed cells to the terminal.
    pub fn render(&mut self, terminal: &Terminal, next: &Buffer) -> io::Result<()> {
        // If buffers sizes are different, clear the screen
        if next.width != self.current_buffer.width || next.height != self.current_buffer.height {
            terminal.write("\x1b[2J".as_bytes())?;
        }

        let diff = next.diff(&self.current_buffer);

        for change in diff {
            terminal.write(format!("\x1b[{};{}H", change.y + 1, change.x + 1).as_bytes())?;
            terminal.write(change.cell.style.to_ansi().as_bytes())?;
            let mut buf = [0u8; 4];
            terminal.write(change.cell.symbol.encode_utf8(&mut buf).as_bytes())?;
        }

        self.current_buffer = next.clone();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, Style, terminal::mocks::MockSystem};

    #[test]
    fn test_renderer_minimal_updates() {
        let mock = MockSystem::new();
        let log_ref = mock.log.clone();
        let terminal = Terminal::new_with_system(Box::new(mock)).unwrap();
        let mut renderer = Renderer::new(3, 3);

        let mut next = Buffer::new(3, 3);
        next.set_with_style(1, 1, 'X', Style::new().fg(Color::Red));

        renderer.render(&terminal, &next).unwrap();

        // Verify that the mock received the correct ANSI codes
        // \x1b[2;2H -> Move to row 2, col 2 (1-indexed)
        // X -> The character
        let log = log_ref.lock().unwrap();
        let found = log.iter().any(|s| s.contains("X"));
        if !found {
            panic!("'X' not found in log: {:?}", log);
        }
        // Check for the style code: Reset(0), Red(31)
        assert!(log.iter().any(|s| s.contains("0;31")));
    }
}
