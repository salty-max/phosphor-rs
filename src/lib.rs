//! **Briks** is a minimalist, testable TUI (Text User Interface) framework for Rust.
//!
//! It follows the **Model-View-Update (MVU)** architecture (similar to Elm or Iced),
//! providing a clean separation between your application logic and the terminal hardware.
//!
//! # Core Concepts
//! * **[`Application`]**: The trait you implement to define your app's state, logic, and view.
//! * **[`Application::Action`]**: A custom type representing things that can happen in your app.
//! * **[`Command`]**: Instructions returned to the runtime (e.g., to quit).
//! * **[`run`]**: The entry point that drives the event loop.
//!
//! # Example
//! ```no_run
//! use briks::{Application, Command, run, input::Event};
//!
//! struct MyApp;
//! impl Application for MyApp {
//!     type Action = ();
//!     fn update(&mut self, _msg: ()) -> Command { Command::Quit }
//!     fn draw(&self) -> String { "Hello Briks!".to_string() }
//! }
//!
//! fn main() -> std::io::Result<()> {
//!     run(MyApp)
//! }
//! ```

use std::io;
use std::thread;
use std::time::Duration;

use crate::{
    input::{Event, Input},
    terminal::Terminal,
};

pub mod input;
#[macro_use]
pub mod logger;
pub mod terminal;

/// Commands returned by the application to control the runtime flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Continue running the application loop.
    None,
    /// Stop the application and exit immediately.
    Quit,
}

/// The core trait for a Briks application.
///
/// Implementors define the state machine and rendering logic for their TUI.
pub trait Application {
    /// The message type used to update the application state.
    ///
    /// This is typically an `enum` representing user actions or system events
    /// that your application cares about.
    type Action;

    /// Called once before the event loop starts.
    ///
    /// Use this to perform any initial setup or return an initial command.
    fn init(&self) -> Command {
        Command::None
    }

    /// Maps a raw terminal [`Event`] to an application-specific [`Self::Action`].
    ///
    /// This method acts as a filter/translator. Return `Some(action)` to trigger
    /// an [`update`](Self::update), or `None` to ignore the event.
    fn on_event(&self, _event: Event) -> Option<Self::Action> {
        None
    }

    /// Updates the application state based on an action.
    ///
    /// This is the only place where you should modify your application state.
    /// It returns a [`Command`] to tell the runtime what to do next.
    fn update(&mut self, msg: Self::Action) -> Command;

    /// Renders the current application state as a string.
    ///
    /// The returned string will be drawn to the terminal. Use ANSI escape codes
    /// for colors and styling, or wait for the upcoming `Buffer` system!
    fn draw(&self) -> String;
}

/// Entry point to run a Briks application.
///
/// This function:
/// 1. Initializes the terminal in **Raw Mode**.
/// 2. Sets up input capturing.
/// 3. Executes the [`Application::init`] hook.
/// 4. Enters the main event loop (Render -> Input -> Update).
///
/// # Errors
/// Returns an [`io::Error`] if the terminal cannot be initialized or if a
/// write operation fails.
pub fn run<App: Application>(app: App) -> io::Result<()> {
    let terminal = Terminal::new()?;
    let input = Input::new();
    run_app(app, terminal, input)
}

/// The internal event loop.
fn run_app<App: Application>(mut app: App, terminal: Terminal, mut input: Input) -> io::Result<()> {
    // Check if the app wants to exit immediately
    if let Command::Quit = app.init() {
        return Ok(());
    }

    loop {
        // --- 1. Render Phase ---
        let view = app.draw();

        // Clear screen (\x1b[2J) and move cursor home (\x1b[H)
        // TODO: Double buffering
        terminal.write(b"\x1b[2J\x1b[H")?;
        terminal.write(view.as_bytes())?;

        // --- 2. Input Phase ---
        let events = input.read(&terminal);
        for event in events {
            // Map raw event -> App Action
            if let Some(msg) = app.on_event(event) {
                // Update State
                match app.update(msg) {
                    Command::Quit => return Ok(()),
                    Command::None => {}
                }
            }
        }

        // --- 3. Idle Phase ---
        // Simple frame limiter (approx 60 FPS) to reduce CPU usage.
        thread::sleep(Duration::from_millis(16));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{Event, KeyCode, KeyEvent};
    // Note: We use the mock system to simulate input without a real terminal
    use crate::terminal::mocks::MockSystem;

    struct TestApp;

    impl Application for TestApp {
        type Action = ();

        fn on_event(&self, event: Event) -> Option<Self::Action> {
            // Quit if 'q' is pressed
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) = event
            {
                Some(())
            } else {
                None
            }
        }

        fn update(&mut self, _msg: Self::Action) -> Command {
            Command::Quit
        }

        fn draw(&self) -> String {
            "Test".to_string()
        }
    }

    #[test]
    fn test_run_loop_quits() {
        // Arrange
        let mock = MockSystem::new();
        mock.push_input(b"q"); // Inject 'q' into the mock input buffer

        // Inject the mock system into the Terminal
        let terminal = Terminal::new_with_system(Box::new(mock)).unwrap();
        let input = Input::new();
        let app = TestApp;

        // Act
        // This runs the loop. It should read 'q', call on_event,
        // receive (), call update, receive Command::Quit, and return Ok.
        let res = run_app(app, terminal, input);

        // Assert
        assert!(res.is_ok());
    }
}
