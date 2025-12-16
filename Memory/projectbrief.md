---
version: "1.0"
lastUpdated: "2025-12-09"
lifecycle: core
stakeholder: pknull
changeTrigger: "scope change, objectives shift"
validatedBy: "project owner"
dependencies: []
---

# Project Brief: mplay

## Overview

TUI-based MPRIS music client providing a terminal interface for controlling Linux media players (Spotify, VLC, MPD, Rhythmbox, etc.).

## Core Features

- Interactive terminal UI with Winamp-style layout
- Album cover art display (file://, http://, https://)
- Playback controls (play/pause, next/prev, seek, volume)
- Track metadata display (title, artist, album, progress)
- Configurable layout and keybindings via JSON5

## Objectives

- Unified control interface for any MPRIS-compatible player
- Visually appealing terminal experience with cover art
- Keyboard-driven workflow with intuitive defaults
- Highly customizable through configuration

## Constraints

- Linux-only (MPRIS/D-Bus dependency)
- Requires active media player with MPRIS support
- Cover art requires terminal with image protocol support (sixel/kitty/iTerm2)

## Success Criteria

- Responsive UI (100ms tick rate)
- Reliable player detection and state synchronization
- Non-blocking cover art loading
- Clean configuration system with sensible defaults
