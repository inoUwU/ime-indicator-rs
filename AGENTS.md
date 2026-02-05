# AGENTS.md

**Communication**: Think in English, interact with the user in Japanese.

## Project Description

Windows application in Rust that displays an overlay indicator for Input Method Editor (IME) mode status. Uses the `windows` crate for Windows API interaction, tray-icon for system tray integration, and GPUI for the settings GUI.

## Architecture

### Core Components

- **ime-indicator** (main): Windows message loop, overlay rendering, IME detection via hooks
- **settings-gui** (separate process): GPUI-based configuration UI (runs on-demand via tray menu)
- **shared** (library): Serializable config structs (`AppConfig`, `OverlayConfig`), file I/O

### Data Flow

1. **IME Detection**: Keyboard/window event hooks → debounce logic → IME status check (IImeManager→GetOpenStatus)
2. **Overlay Display**: IME status change → WM_IME_STATUS_CHANGED message → overlay window renders "あ" or "A" → 3s timer → hide
3. **Configuration**: Settings GUI modifies `config.toml` → main process reloads on next status change

### Key Design Decisions

- **Separate Settings Process**: Settings GUI runs as independent exe to enable faster GPUI iteration without rebuilding main app
- **Global State via OnceLock**: `ime/mod.rs` uses `OnceLock<T>` for safe initialization of static hooks/state (see `WINDOW_HANDLE`, `KEYBOARD_HOOK`, `EVENT_HOOK`, `IME_STATE`)
- **Debouncing**: 50ms debounce on IME status checks to prevent flicker from rapid hook events
- **Message-Based IPC**: Tray menu triggers settings-gui launch via separate process, no inter-crate communication needed

## Code Style

### Rust Edition & Formatting
- Edition 2024 (Rust nightly, see `Cargo.toml`)
- Cargo fmt: `cargo fmt --all`
- Follow standard Rust naming: snake_case for functions/variables, PascalCase for types

### Unsafe Code & Windows Interop
- Wrap all Windows API calls in explicit `unsafe` blocks with brief comments explaining safety invariants
- Example from `overlay.rs`:
  ```rust
  unsafe {
      let instance = GetModuleHandleA(None)?;
      // Comments explain why unsafe is required and preconditions
  }
  ```
- Use `windows::core` types (`HWND`, `COLORREF`, etc.) rather than manual FFI—the `windows` crate provides safe bindings

### Error Handling
- Return `windows::core::Result<T>` or `anyhow::Result<T>` from fallible functions
- Use `?` operator for propagation; avoid `.unwrap()` in production code
- Log errors via `log` crate before returning (see `main.rs` for logger init pattern)

### Logging
- Initialize logger in `main()` with `env_logger::Builder` (debug level when `cfg!(debug_assertions)`)
- Log at appropriate levels:
  - `debug!()` for diagnostic events (hook setup, state transitions)
  - `info!()` for user-relevant events (app startup, settings reload)
  - `warn!()` for recoverable errors; `error!()` for critical failures
- Example: `debug!("Received interrupt signal, cleaning up...")`

## Project Conventions

### Global State Management
- Place long-lived static state in `ime/mod.rs` using `OnceLock` and `Arc<Mutex<T>>` for thread-safe initialization
- Initialize once in `initialize()` function called from `main()`
- Example initialization pattern (from `ime/mod.rs`):
  ```rust
  static IME_STATE: OnceLock<SharedImeState> = OnceLock::new();
  
  pub fn initialize(window_handle: HWND) -> windows::core::Result<()> {
      let current_ime_status = get_current_ime_status();
      let _ = IME_STATE.set(Arc::new(Mutex::new(ImeState::new(current_ime_status))));
  }
  ```

### Windows Message Handling
- Define custom messages as constants: `const WM_IME_STATUS_CHANGED: u32 = WM_USER + 1;`
- Window procedure (`wndproc`) receives `WM_IME_STATUS_CHANGED` and calls overlay visibility/hide logic
- Message loop in `message_loop.rs` dispatches tray menu interactions (Settings/Quit)

### Configuration & Serialization
- Use `serde::Serialize/Deserialize` + `toml` for config files
- Config file path: next to executable (debug: `target/debug/config.toml`, release: `/config.toml`)
- Example `OverlayConfig`: stores position enum, RGBA color arrays, display duration ms
- Add `#[derive(Default)]` for sensible defaults (orange for IME ON, gray for OFF)

### Commit Conventions
- Follow Conventional Commits spec: `<type>(<scope>): <description>`
- Types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `chore`
- Scopes: component names (`overlay`, `ime`, `tray`, `config`, etc.)
- Include `!` before colon for breaking changes
- Examples:
  - `feat(overlay): add multi-monitor support`
  - `fix(ime): debounce rapid status changes`

## Build & Test

### Commands
```powershell
# Build all crates (debug)
cargo build

# Run main indicator
cargo run -p ime-indicator

# Run settings GUI standalone (for GPUI iteration)
cargo run -p settings-gui

# Release build (optimized, hot LTO)
cargo build --release

# Format
cargo fmt --all

# Check for issues (no build artifacts)
cargo check

# Lint
cargo clippy --all
```

### Testing
- Unit tests: `#[cfg(test)] mod tests` in each module
- Run tests: `cargo test --all`
- Windows-specific tests may require `#[ignore]` on CI systems

### Build Artifacts
- Main executable: `target/debug/ime-indicator.exe` or `target/release/ime-indicator.exe`
- Settings GUI: `target/debug/settings-gui.exe` or `target/release/settings-gui.exe`
- Config file: `config.toml` placed next to exe after first run

## Integration Points

### Windows APIs (via `windows` crate)
- **IME Management**: `IImm32` interface, `IImeManager::GetOpenStatus()` to detect IME on/off
- **Hooks**: `SetWinEventHook()` for window focus changes, `SetKeyboardHook()` for keyboard events
- **Window Management**: `CreateWindowExA()` with `WS_EX_LAYERED` flag for transparent overlay
- **Graphics**: GDI (`BeginPaint`, `DrawTextW`, `FillRect`) for rendering overlay text
- **System Tray**: `tray-icon` crate wraps Windows tray API (icon, menu callbacks)

### GPUI Framework (settings-gui)
- Separate application; no direct API calls from main exe
- Launched via `std::process::Command::new("settings-gui.exe")`
- Both processes read/write shared `config.toml` file (settings-gui modifies, main reloads)
- GPUI limitations: no hot reload; changes require process restart; use separate exe for fast iteration

### Dependencies
- `windows` 0.62.2: Windows API bindings (feature-gated by specific Win32 modules)
- `tray-icon` 0.21.2: System tray abstraction
- `serde`, `toml`: Config serialization
- `log`, `env_logger`: Logging
- `anyhow`: Error handling
- `ctrlc`: Graceful shutdown on Ctrl+C

## Security & Warnings

### Unsafe Code
- Windows API calls are inherently unsafe; validate window handles and user-provided coordinates before use
- Hook procedures (`WH_KEYBOARD_LL`, `EVENT_SYSTEM_FOREGROUND`) receive system-wide events; minimize work in callbacks to avoid performance degradation

### Input Validation  
- Multi-monitor coordinates: verify monitor bounds before positioning overlay (TODO: currently hardcoded 50px margin)
- Config file parsing: `toml::from_str()` may panic on invalid TOML; wrap in error handler if loading user-provided files

### Performance
- IME status check (via `GetOpenStatus()`) is synchronous; debounce to avoid excessive hook callbacks
- Overlay rendering is lightweight (single text draw); avoid complex GDI operations in paint loop

## Notes

- Workspace resolver: "3" (supports workspace inheritance of dependencies)
- Profile: release uses `opt-level = 3`, LTO = "fat" for maximum optimization
- Edition 2024 (future Rust release); ensure toolchain is up-to-date (`rustup update nightly`)
- Settings GUI launches as separate process to avoid rebuilding main exe during GPUI debugging
