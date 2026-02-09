//! The `buffer` module provides a 2D grid of cells for efficient rendering.
//!
//! A [`Buffer`] represents a single frame of the TUI. By comparing two buffers,
//! the framework can perform "diff-rendering," only updating the parts of the
//! terminal that have actually changed.

/// A single character on the screen with its associated style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    /// The character to display in this cell.
    pub symbol: char,
    // TODO: Add Style (foreground, background, modifiers)
}

impl Default for Cell {
    /// Returns a cell containing a space character.
    fn default() -> Self {
        Self { symbol: ' ' }
    }
}

/// Represents a single cell change between two frames.
#[derive(Debug, PartialEq, Eq)]
pub struct Change {
    pub x: u16,
    pub y: u16,
    pub cell: Cell,
}

/// A 2D grid of [`Cell`]s representing a terminal frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
    /// The width of the buffer in columns.
    pub width: u16,
    /// The height of the buffer in rows.
    pub height: u16,
    /// The linear storage of cells (row-major order).
    pub content: Vec<Cell>,
}

impl Buffer {
    /// Creates a new buffer of the given size, filled with default cells.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            content: vec![Cell::default(); (width * height) as usize],
        }
    }

    /// Returns a reference to the cell at the given coordinates.
    ///
    /// # Panics
    /// Panics if the coordinates are out of bounds.
    pub fn get(&self, x: u16, y: u16) -> &Cell {
        if x >= self.width || y >= self.height {
            panic!("Index out of bounds!")
        }

        self.content
            .get(self.index(x, y))
            .expect("No cell found at {x}:{y}")
    }

    /// Sets the character at the given coordinates.
    ///
    /// Does nothing if the coordinates are out of bounds.
    pub fn set(&mut self, x: u16, y: u16, symbol: char) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = self.index(x, y);
        self.content[idx].symbol = symbol;
    }

    /// Helper to convert 2D coordinates to a 1D index.
    fn index(&self, x: u16, y: u16) -> usize {
        ((y * self.width) + x) as usize
    }

    /// Compares this buffer with another and returns the list of changed cells.
    ///
    /// This is used to perform minimal updates to the terminal.
    pub fn diff(&self, other: &Buffer) -> Vec<Change> {
        let mut changes: Vec<Change> = Vec::new();

        if self.width != other.width || self.height != other.height {
            return self
                .content
                .iter()
                .enumerate()
                .map(|(i, cell)| Change {
                    x: (i as u16) % self.width,
                    y: (i as u16) / self.width,
                    cell: *cell,
                })
                .collect();
        } else {
            for (i, (new_cell, old_cell)) in
                self.content.iter().zip(other.content.iter()).enumerate()
            {
                if new_cell != old_cell {
                    changes.push(Change {
                        x: (i as u16) % self.width,
                        y: (i as u16) / self.width,
                        cell: *new_cell,
                    })
                }
            }
        }

        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_initialization() {
        let buf = Buffer::new(10, 5);
        assert_eq!(buf.width, 10);
        assert_eq!(buf.height, 5);
        assert_eq!(buf.content.len(), 50);
        assert_eq!(buf.get(0, 0).symbol, ' ');
    }

    #[test]
    fn test_buffer_set_get() {
        let mut buf = Buffer::new(10, 5);
        buf.set(2, 3, 'X');
        assert_eq!(buf.get(2, 3).symbol, 'X');
        assert_eq!(buf.get(0, 0).symbol, ' ');
    }

    #[test]
    #[should_panic]
    fn test_buffer_get_out_of_bounds() {
        let buf = Buffer::new(10, 5);
        buf.get(10, 5);
    }

    #[test]
    fn test_buffer_diff() {
        let old = Buffer::new(3, 3);
        let mut new = Buffer::new(3, 3);
        new.set(1, 1, 'X');
        new.set(2, 2, 'Y');

        let changes = new.diff(&old);
        assert_eq!(changes.len(), 2);
        assert_eq!(
            changes[0],
            Change {
                x: 1,
                y: 1,
                cell: Cell { symbol: 'X' }
            }
        );
        assert_eq!(
            changes[1],
            Change {
                x: 2,
                y: 2,
                cell: Cell { symbol: 'Y' }
            }
        );
    }

    #[test]
    fn test_buffer_diff_size_mismatch() {
        let old = Buffer::new(2, 2);
        let mut new = Buffer::new(3, 3);
        new.set(0, 0, 'A');

        let changes = new.diff(&old);
        // Should return all 9 cells of the new buffer
        assert_eq!(changes.len(), 9);
        assert_eq!(changes[0].cell.symbol, 'A');
        assert_eq!(changes[1].cell.symbol, ' ');
    }
}
