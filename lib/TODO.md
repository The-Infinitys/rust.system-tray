# TODO List

## High Priority

- [ ] Implement the non-blocking (asynchronous) API in `qt6.rs`.
  - [ ] Implement `QtApp::start()` to run the Qt event loop in a separate thread.
  - [ ] Implement `QtAppInstance` to manage the running application.
  - [ ] Implement `QtAppEvent` enum and the `poll_event` C API to send events (like tray clicks) from C++ to Rust.
  - [ ] Create an event listener or callback mechanism in `qt6.rs` for handling these events.

## Low Priority

- [ ] Investigate the feasibility of dynamically building `QWidget` components from Rust. This is a complex research task.
