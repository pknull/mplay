---
version: "1.0"
lastUpdated: "2025-12-09"
lifecycle: core
stakeholder: pknull
changeTrigger: "tooling changes, dependency updates"
validatedBy: "build verification"
dependencies: []
---

# Tech Environment

## Language & Build

- **Language**: Rust (Edition 2021)
- **Toolchain**: Rust stable
- **Package Manager**: Cargo

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| ratatui | 0.29 | TUI framework |
| crossterm | 0.28 | Terminal abstraction, input |
| mpris | 2.0 | D-Bus media player control |
| image | 0.25 | Image loading |
| ratatui-image | 4 | Cover art rendering |
| serde | 1.0 | Serialization |
| serde_json | 1.0 | JSON parsing |
| json5 | 0.4 | JSON5/JSONC support |
| ureq | 2.9 | HTTP client for remote covers |
| directories | 5.0 | Config path resolution |
| anyhow | 1.0 | Error handling |
| urlencoding | 2.1 | URL encoding |

## Build Commands

```bash
cargo build --release    # Optimized build (LTO, stripped)
cargo run                # Debug execution
./target/release/mplay   # Run release binary
```

## Project Structure

```
src/
├── main.rs           # Entry point, init
├── config.rs         # Configuration system (403 lines)
├── mpris_client.rs   # MPRIS protocol wrapper (259 lines)
├── cover.rs          # Cover art loading (120 lines)
└── ui/
    ├── mod.rs        # Module exports
    ├── app.rs        # Main event loop (248 lines)
    └── widgets.rs    # Layout & widget rendering (481 lines)
```

## Configuration

**Location**: `~/.config/mplay/config.json`

**Format**: JSON5 (comments supported)

**Key Sections**:
- `players`: Priority list of player names
- `keybinds`: Key mappings for all actions
- `layout`: Recursive container/widget structure
- `widgets`: Widget-specific styling

## Code Conventions

- snake_case for functions/variables
- PascalCase for types
- `anyhow::Result<T>` for error propagation
- `.context()` for error messages
- Enum-based config with serde rename_all
- Early returns for error handling

## Key Patterns

- Non-blocking cover loading (thread + mpsc)
- Constraint-based recursive layout
- State polling (500ms) vs UI tick (100ms)
- Configuration-driven layout/styling
- Graceful degradation (missing metadata → "Unknown")

## Default Keybindings

| Key | Action |
|-----|--------|
| q, Esc | Quit |
| Space | Play/pause |
| n, Right | Next track |
| p, Left | Previous track |
| l, Shift+Right | Seek forward |
| h, Shift+Left | Seek backward |
| k, Up | Volume up |
| j, Down | Volume down |

## Widget Types

- `Label`: Text display (title, artist, album)
- `Progress`: Seekable progress bar
- `Volume`: Volume indicator
- `Button`: Clickable controls
- `CoverArt`: Album artwork
- `Spectrum`: Audio visualization (placeholder)
- `Empty`: Spacer
