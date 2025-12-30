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
- Settings for indicator position, size (planned)

## Project Structure

```text
src/
├── main.rs              # Entry point - initializes and integrates all modules
├── tray.rs              # System tray management (TrayManager, icon, menu)
├── ime/
│   ├── mod.rs           # IME detection logic, hooks setup, debouncing
│   └── state.rs         # IME state structure (ImeState, SharedImeState)
├── overlay.rs           # Overlay window creation, rendering, show/hide
├── message_loop.rs      # Windows message loop + tray event integration
└── utils/
    └── string_utils.rs  # Utility functions
```

## Module Responsibilities

### main.rs

- Application entry point
- Creates overlay window
- Initializes IME manager
- Sets up system tray
- Configures Ctrl+C handler
- Runs message loop
- Cleans up resources on exit

### tray.rs

- `TrayManager` struct managing system tray lifecycle
- Menu creation (Quit item)
- Icon generation (32x32 red circle)
- Exposes `quit_menu_id` for event handling

### ime/mod.rs

- IME state detection using Windows IMM APIs
- Low-level keyboard hook (`WH_KEYBOARD_LL`)
- Window event hook for focus changes
- Debouncing to prevent excessive checks
- Sends `WM_IME_STATUS_CHANGED` message on state changes
- Global state management with `OnceLock` and `Mutex`

### ime/state.rs

- `ImeState` struct: tracks active status, description, update time
- `SharedImeState` type alias for thread-safe state sharing
- Helper methods for state initialization and updates

### overlay.rs

- Creates layered, transparent, topmost window
- Window procedure handling WM_PAINT, WM_TIMER, WM_DESTROY
- Renders IME status ("あ" for active, "A" for inactive)
- Color-coded backgrounds (orange for active, gray for inactive)
- 3-second auto-hide timer

### message_loop.rs

- Non-blocking message loop using `PeekMessageA`
- Checks tray menu events
- Monitors quit flag (`AtomicBool`)
- Dispatches Windows messages
- Graceful shutdown on quit

## Key Technologies

- **windows-rs**: Windows API bindings
- **tray-icon**: System tray integration
- **ctrlc**: Ctrl+C signal handling
- Low-level hooks for IME monitoring
- Layered windows for transparency

## Build & Run

```powershell
cargo build
cargo run
```

## Notes

- Window Event Hook setup may fail (non-critical, backup timer compensates)
- Keyboard hook requires appropriate permissions
- Overlay positioned at top-right (50px margins)
