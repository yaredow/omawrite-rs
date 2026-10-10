# Omawrite (Rust) ✍️

A fast, distraction-free Markdown writing tool built natively for Linux with Rust, GTK4, Libadwaita, and Relm4.

Inspired by [Omawrite](https://github.com/omacom/omawrite) from the Omarchy desktop environment, rewritten in Rust for memory safety, minimal resource consumption, and single-binary portability.

---

## Features

- **Zen Writing Canvas**: Clean, centered text area with distraction-free layout.
- **Native Desktop Integration**: Automatically adapts to system dark/light themes and display scaling via Libadwaita.
- **Live Markdown Syntax Highlighting**: Powered by `GtkSourceView5`.
- **Rock-Solid File Safety**:
  - Atomic saves preventing file corruption on abrupt shutdowns.
  - Periodic background crash recovery drafts.
  - External file change notifications to prevent overwriting edits.
- **Writer Metrics**: Real-time debounced word count and reading time.
- **Keyboard-First**: Keyboard shortcuts for Markdown syntax (`Ctrl+B`, `Ctrl+I`, `Ctrl+K`), quick search, and fullscreen focus.

---

## Architecture & Tech Stack

- **Language**: Rust (2021 Edition)
- **GUI & Application**: [Relm4](https://relm4.org/) (Elm-inspired reactive framework for GTK4)
- **Interface & Themes**: [Libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/) + GTK4
- **Editor Canvas**: [GtkSourceView5](https://gitlab.gnome.org/GNOME/gtksourceview)
- **File System Monitoring**: `notify` crate
- **Safe I/O**: `tempfile` (atomic writes) and `fd-lock` (draft locking)
- **Environment & Packaging**: Nix Flakes (`flake.nix`)

---

## Getting Started

### Using Nix Flakes (Recommended)

Ensure Nix with flakes is installed.

1. Enter the development environment:
   ```bash
   nix develop
   ```
   *(Or allow `direnv` with `direnv allow` to automatically load the shell).*

2. Run the application:
   ```bash
   cargo run
   ```

### Standard Linux Build

Ensure GTK4, Libadwaita, and GtkSourceView5 development libraries are installed on your distribution:
```bash
cargo build --release
./target/release/omawrite-rs
```

---

## License

MIT License. See [LICENSE](LICENSE) for details.
