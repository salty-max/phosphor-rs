//! A simple, thread-safe file logger for debugging TUI applications.
//!
//! Because TUI applications capture `stdout` and `stderr` for rendering,
//! standard `println!` debugging is invisible or breaks the UI.
//! This module redirects logs to a file (`debug.log`).
//!
//! # Example
//! ```no_run
//! use briks::logger;
//!
//! fn main() -> std::io::Result<()> {
//!     logger::init()?;
//!     briks::log!("Application started");
//!     Ok(())
//! }
//! ```

use std::fs::File;
use std::io::{self, Write};
use std::sync::Mutex;

/// Global singleton for the log file handle.
///
/// We use a `Mutex` to ensure safe concurrent access from multiple threads.
static LOGGER: Mutex<Option<File>> = Mutex::new(None);

/// Initializes the logger, creating (or truncating) `debug.log`.
///
/// This should be called once at the start of the application.
///
/// # Errors
/// Returns an [`io::Error`] if the file cannot be created.
pub fn init() -> io::Result<()> {
    let file = File::create("debug.log")?;
    let mut guard = LOGGER.lock().unwrap();
    *guard = Some(file);
    Ok(())
}

/// Internal function to write a formatted string to the log file.
///
/// Prefer using the [`crate::log!`] macro instead of calling this directly.
pub fn write_log(msg: &str) {
    if let Ok(mut guard) = LOGGER.lock()
        && let Some(file) = guard.as_mut()
    {
        // We ignore write errors to prevent panics in the logging infrastructure
        let _ = writeln!(file, "{}", msg);
    }
}

/// Logs a message to the `debug.log` file.
///
/// Usage is identical to `println!`.
///
/// # Example
/// ```no_run
/// # use briks::log;
/// log!("Value: {}", 42);
/// ```
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::logger::write_log(&format!($($arg)*));
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_logging() {
        // Initialize
        // Note: In a real test suite, this might conflict with other tests
        // using the same file.
        init().unwrap();

        // Log something
        log!("Hello {}", "World");
        log!("Another line");

        // Force the lock to drop so the file flushes (OS dependent, but likely safe here)

        // Verify content
        let content = fs::read_to_string("debug.log").unwrap();
        assert!(content.contains("Hello World"));
        assert!(content.contains("Another line"));

        // Cleanup
        let _ = fs::remove_file("debug.log");
    }
}
