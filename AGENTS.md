# AGENTS.md

- Think in English, interact with the user in Japanese.

## Project Description

This project is a Windows application written in Rust that displays a overlay indicator for Input Method Editors (IMEs) Mode.
It using the `windows` crate to interact with Windows APIs.

## Features

- Displays an overlay indicator for IME mode using windows hooks
- System tray integration with Quit menu
- Automatic overlay display (3 seconds) on IME status changes
- Keyboard and window event hooks for real-time IME detection
- Settings GUI (separate executable) using GPUI

## Project Structure (Cargo Workspace)

```text
ime-indicator-rs/
├── Cargo.toml              # Workspace root
├── assets/
│   └── icon.ico
├── crates/
│   ├── ime-indicator/      # Main application (overlay, tray, hooks)
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   └── src/
│   │       ├── main.rs
│   │       ├── message_loop.rs
│   │       ├── overlay.rs
│   │       ├── tray.rs
│   │       └── ime/
│   │           ├── mod.rs
│   │           └── state.rs
│   ├── settings-gui/       # Settings window (GPUI, separate exe)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── gui.rs
│   └── shared/             # Shared config structs
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── config.rs
```

## Module Responsibilities

### crates/ime-indicator (main app)

- **main.rs**: Entry point, initializes overlay, IME hooks, tray, message loop
- **tray.rs**: System tray management (TrayManager, icon, Settings/Quit menus)
- **overlay.rs**: Layered transparent overlay window, renders "あ"/"A"
- **message_loop.rs**: Windows message loop, launches settings-gui.exe
- **ime/mod.rs**: IME detection via hooks, debouncing, WM_IME_STATUS_CHANGED
- **ime/state.rs**: ImeState struct, SharedImeState type

### crates/settings-gui (settings window)

- **main.rs**: Entry point, calls gui::show()
- **gui.rs**: GPUI-based settings UI (TODO: input fields)

### crates/shared (common library)

- **config.rs**: AppConfig, OverlayConfig structs, load/save functions

## Build & Run

```powershell
# Build all crates
cargo build

# Run main indicator
cargo run -p ime-indicator

# Run settings GUI only (for GPUI practice)
cargo run -p settings-gui

# Release build
cargo build --release
```

## Notes

- Settings GUI is launched as a separate process from tray menu
- GPUI does not support HOT reload, but separate exe allows faster iteration
- Shared config stored in `config.toml` next to executable
