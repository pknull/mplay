---
version: "1.0"
lastUpdated: "2025-12-09"
lifecycle: core
stakeholder: pknull
changeTrigger: "workflow pattern changes"
validatedBy: "usage verification"
dependencies: ["techEnvironment.md"]
---

# Workflow Protocols

## Development Workflow

1. **Feature Addition**
   - Determine if config change needed
   - Add widget type if new UI element
   - Implement in appropriate module
   - Test with multiple players

2. **Widget Development**
   - Add variant to widget enum in `config.rs`
   - Implement rendering in `widgets.rs`
   - Add constraint calculation
   - Support styling options

3. **MPRIS Integration**
   - Add method to `MprisClient`
   - Handle D-Bus errors gracefully
   - Update `PlayerState` if new metadata

## Testing Protocol

- Manual testing with Spotify, VLC, MPD
- Verify cover art loading (local, HTTP, HTTPS)
- Test terminal resize handling
- Check keybinding responsiveness
- Verify clean exit

## Configuration Changes

When modifying config schema:
1. Update struct in `config.rs`
2. Add serde attributes
3. Update `default()` implementation
4. Test config load/save roundtrip
5. Document in README (when exists)

## Performance Considerations

- Cover art loading must be non-blocking
- State polling at 500ms balances responsiveness/CPU
- UI tick at 100ms for smooth interaction
- Cache cover art to avoid repeated fetches
- Minimize allocations in render loop

## Adding New Widget Types

1. Add variant to widget enum (`config.rs`)
2. Add constraint calculation in `render_layout()` (`widgets.rs`)
3. Implement rendering in `render_widget()` (`widgets.rs`)
4. Support common styling (colors, alignment)
5. Add area tracking if interactive (clicks)
6. Update default config if sensible default exists
