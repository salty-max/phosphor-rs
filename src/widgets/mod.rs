//! The `widgets` module provides reusable UI components.

use crate::{Frame, Rect};

pub mod block;
pub mod list;
pub mod scrollable;
pub mod text;

pub use block::{Block, BorderType, Borders};
pub use list::List;
pub use scrollable::Scrollable;
pub use text::Text;

/// The core trait for all UI components.
pub trait Widget {
    /// Draws the widget into the given area of the frame.
    fn render(self, area: Rect, frame: &mut Frame);
}
