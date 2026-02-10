# Phosphor Project Plan 🧱

This document tracks the progress and future roadmap of the **Phosphor** TUI framework.

## ✅ Completed Phases

### Phase 1: Hardware Abstraction Layer (HAL)
- [x] Define `System` trait for OS abstraction.
- [x] Implement `LibcSystem` for production.
- [x] Implement `MockSystem` for unit testing.
- [x] Create `Terminal` RAII wrapper.
- [x] Implement robust cleanup (cursor restoration, alternate buffer exit).

### Phase 2: Input Handling
- [x] Implement ANSI escape sequence parser.
- [x] Support multi-byte UTF-8 characters.
- [x] Implement non-blocking polling for ambiguous keys (Esc vs Alt).
- [x] Add `Input` reader with fragmented sequence support.

### Phase 3: Rendering Engine
- [x] Implement 2D `Buffer` with `Cell` storage.
- [x] Build `Diff` engine to identify changed cells.
- [x] Create `Renderer` to translate diffs into ANSI escape codes.
- [x] Implement "Big Bang" initial clear to sync terminal state.

### Phase 4: Styling System
- [x] Support ANSI 16, 256, and TrueColor (RGB).
- [x] Implement Hex color parsing.
- [x] Add text modifiers (Bold, Italic, Underline, etc.).
- [x] Create stateful `Frame` API for styled drawing.

### Phase 5: Layout & Composition
- [x] Define `Rect` primitive.
- [x] Implement `Layout` engine with `Length`, `Percentage`, `Ratio`, and `Fill`.
- [x] Add `split_to` helper for ergonomic area destructuring.
- [x] Implement `Frame::render_area` for nested sub-frames.

### Phase 6: Initial Widgets
- [x] Define `Widget` trait.
- [x] Implement `Text` widget with styling.
- [x] Implement `Block` widget with multiple border types and padding.

---

## 🚧 Current Work
- [ ] **Advanced Layouts**: Implement `Min` and `Max` constraints.
- [ ] **Widget Library**: Add `List`, `Gauge`, and `Table`.

---

## 🚀 Future Roadmap

### Phase 7: Advanced Interaction & Polish
- [ ] **Mouse Support**: Enable terminal mouse tracking and parse click/scroll events.
- [ ] **Diff-Styling**: Optimize renderer to only send style codes when they change.
- [ ] **Buffer Swapping**: Use double-buffering to further reduce flicker.
- [ ] **Panic Hook**: Ensure terminal is restored even if the app panics.

### Phase 8: Advanced Widgets
- [ ] **List**: Scrollable list of items.
- [ ] **Gauge**: Progress bar.
- [ ] **Table**: Multi-column data display.
- [ ] **Input Field**: Single-line text input.

### Phase 9: Ecosystem
- [ ] **Documentation**: Complete API docs and tutorials.
- [ ] **Examples**: Add more complex demos (e.g., a Git client or System Monitor).
- [ ] **Crate Publication**: Prepare for crates.io.
