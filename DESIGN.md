# Phosphor - TUI Framework Design

## 1. Philosophy

**Phosphor** is a state-driven TUI (Text User Interface) framework for Rust.

- **Core principle**: The UI is a pure function of state. All rendering is derived from an immutable snapshot; side effects are modeled explicitly.
- **Constraint**: From scratch. No external dependencies other than `libc` for system calls.
- **Methodology**: Test-Driven Development using Dependency Injection to isolate system interactions.

## 2. Architecture

The framework uses a unidirectional data flow:

```
Event -> update(State, Event) -> (State, Command) -> view(State) -> render to terminal
```

### Application Trait

```rust
pub trait Application {
    type Message;

    /// Returns the initial command to execute on startup (e.g., enable raw mode).
    fn init(&self) -> Command<Self::Message>;

    /// Processes a message and transitions state. Returns the next command to run.
    fn update(&mut self, msg: Self::Message) -> Command<Self::Message>;

    /// Renders the current state into a string for display.
    fn view(&self) -> String;
}
```

### Commands

A `Command` represents a side effect to be executed by the runtime, not by the application logic directly. This keeps `update` pure and testable.

```rust
pub enum Command<Msg> {
    /// Do nothing.
    None,
    /// Exit the application.
    Quit,
    /// Execute an arbitrary side effect that may produce a message.
    Perform(Box<dyn FnOnce() -> Option<Msg>>),
    /// Run multiple commands.
    Batch(Vec<Command<Msg>>),
}
```

The runtime executes commands after each `update` call, feeding any resulting messages back into the loop.

## 3. Module Structure

### A. `terminal` - Hardware Abstraction Layer

Handles all direct interaction with the OS. Uses a trait to mock system calls in tests.

```rust
pub trait System {
    fn open_tty(&self) -> Result<RawFd>;
    fn enable_raw_mode(&self, fd: RawFd) -> Result<Termios>;
    fn disable_raw_mode(&self, fd: RawFd, original: &Termios) -> Result<()>;
    fn get_window_size(&self, fd: RawFd) -> Result<(u16, u16)>;
    fn read(&self, fd: RawFd, buf: &mut [u8]) -> Result<usize>;
    fn write(&self, fd: RawFd, buf: &[u8]) -> Result<usize>;
}
```

- `LibcSystem` - Production implementation using `libc` (unsafe).
- `MockSystem` - Test double that captures calls and returns deterministic values.
- `Terminal` - High-level wrapper accepting `Box<dyn System>`. Manages raw mode lifecycle via `Drop`.

### B. `input` - Event Processing

Converts raw byte streams from stdin into semantic events.

```rust
pub enum Event {
    Key(KeyEvent),
    Resize(u16, u16),
}

pub enum KeyEvent {
    Char(char),
    Enter,
    Backspace,
    Escape,
    Up, Down, Left, Right,
    // ...
}
```

- **Reader**: Non-blocking polling loop over the TTY file descriptor.
- **Parser**: State machine that classifies raw bytes and ANSI escape sequences into `Event` values.
  - `\x1b[A` -> `Event::Key(Up)`
  - `\x1b[B` -> `Event::Key(Down)`
  - `a` -> `Event::Key(Char('a'))`

### C. `rendering` - Output Engine

Translates `view()` output into terminal writes.

- **V1 (MVP)**: Immediate mode. Clear screen (`\x1b[2J`) + reset cursor (`\x1b[H`) + write full buffer.
- **V2**: Differential rendering. Retains the previous frame buffer, diffs against the current frame, and emits the minimal set of ANSI escape sequences to update the screen.

## 4. Testing Strategy

System calls cannot be verified in a standard test harness. Dependency injection solves this:

| Layer | Strategy |
|---|---|
| `terminal` | `Terminal::new(Box<dyn System>)` - inject `MockSystem` in tests, `LibcSystem` in production |
| `input` parser | Pure unit tests - feed byte slices in, assert `Event` values out |
| `rendering` diff | Pure unit tests - compare previous and current frame buffers, assert correct ANSI output |
| `Application` logic | Unit test `update` directly - pass messages in, assert state + commands out |

## 5. Development Roadmap

### Phase 1: Foundation
- [x] Implement `System` trait and `LibcSystem` (raw mode, window size, read/write)
- [x] Implement `Terminal` wrapper with `Drop`-based cleanup
- [x] Verify raw mode toggle with `MockSystem` tests

### Phase 2: Input & Events
- [x] Define `Event` and `KeyEvent` enums
- [x] Implement non-blocking stdin reader
- [x] Build ANSI escape sequence parser

### Phase 3: Runtime
- [x] Define `Application` trait and `Command` enum
- [x] Implement the main event loop (`run` function)
- [x] Wire up: poll events -> `update` -> execute commands -> `view` -> render

### Phase 4: Layout & Styling
- [ ] Create ANSI style builder (colors, bold, underline, reset)
- [ ] Implement box-model layout engine (fixed sizes, flex rows/columns)
