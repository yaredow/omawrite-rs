# AGENT.md

Developer & AI Agent guide for `omawrite-rs`.

## Overview

`omawrite-rs` is a lightweight, distraction-free Markdown writing tool for Linux written in Rust. It re-implements Omarchy's `omawrite` (originally C++/Qt6) using GTK4, Libadwaita, GtkSourceView5, and Relm4.

## Environment & Build Commands

- **Environment**: Managed via Nix Flakes. Run commands inside `nix develop` or prefix with `nix develop --command`.
- **Compile & Typecheck**:
  ```bash
  nix develop --command cargo check
  ```
- **Run**:
  ```bash
  nix develop --command cargo run
  ```
- **Test**:
  ```bash
  nix develop --command cargo test
  ```
- **Format & Lint**:
  ```bash
  nix develop --command cargo fmt --check
  nix develop --command cargo clippy
  ```

## Architecture & Module Responsibilities

```text
src/
├── main.rs     # Application entry point, RelmApp setup, CLI argument handling
├── app.rs      # Main UI window, toolbar/headerbar, footer status bar, view! macro
├── editor.rs   # GtkSourceView editor bindings, word counting, Markdown shortcuts
├── storage.rs  # Atomic file saving (tempfile), draft recovery, and lockfiles (fd-lock)
├── watcher.rs  # Background disk watcher (notify) detecting external file modifications
└── theme.rs    # Custom typography (IBM Plex Mono) and CSS tweaks
```

## Engineering Rules for Agents

1. **Non-blocking UI**: Never perform file I/O or background disk watching directly on the main GTK thread. Use channels (`relm4::Sender` / `glib::MainContext`) for async notifications.
2. **Atomic Writes Only**: Always write through a temporary file in the same directory before renaming to prevent data loss or partial writes during crashes.
3. **Draft Recovery**: Periodically write unsaved editor contents to `.omawrite~<hash>.draft` with an advisory lock file (`fd-lock`).
4. **Clean Relm4 State**: Keep state in `AppModel`. UI events must flow through `AppMsg` variants rather than ad-hoc widget mutation callbacks where possible.
5. **Zero Compiler Warnings**: All code changes must compile with `cargo check` cleanly without unused variables or deprecated functions.
