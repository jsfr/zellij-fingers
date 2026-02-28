# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

zellij-fingers is a **Zellij WASM plugin** written in **Rust** that highlights pattern matches (URLs, file paths, git SHAs, IPs, etc.) in Zellij panes and lets users select them via Huffman-encoded keyboard hints.

## Build & Development

Requires Rust with `wasm32-wasip1` target.

```bash
# Install WASM target
rustup target add wasm32-wasip1

# Build WASM plugin (default target via .cargo/config.toml)
cargo build

# Production build
cargo build --release

# Run tests (on host target, not WASM)
cargo test --target "$(rustc -vV | grep host | awk '{print $2}')"

# Build and install plugin to ~/.config/zellij/plugins/
just install
```

The `.cargo/config.toml` sets the default build target to `wasm32-wasip1`. Tests must be run with an explicit host target since WASM doesn't support test harnesses. The plugin is built as a binary (`src/main.rs`), not a cdylib — Zellij requires a `_start` export which only binaries provide.

## Architecture

**Plugin lifecycle:** `load()` parses KDL config, requests permissions, subscribes to events → `update()` drives a state machine (WaitingForPermissions → Capturing → Hinting → Done) → `render()` outputs ANSI-formatted content.

**Core flow:** Plugin opens as floating pane → captures target pane content via `dump-screen` → `Hinter` scans patterns and generates Huffman hints → `renderer` outputs hints with ANSI formatting → keyboard events drive hint selection → `action` executes copy/open/paste via `run_command`.

### Key modules

| File | Purpose |
|------|---------|
| `src/main.rs` | Plugin entry point, `ZellijPlugin` trait impl, state machine |
| `src/config.rs` | Configuration from KDL, `BUILTIN_PATTERNS`, `ALPHABET_MAP` |
| `src/hinter.rs` | Pattern matching engine, hint assignment, tab/width correction |
| `src/huffman.rs` | Huffman tree-based hint generation |
| `src/priority_queue.rs` | Min-heap priority queue for Huffman |
| `src/match_formatter.rs` | ANSI styling for hints overlaid on matches |
| `src/renderer.rs` | Compose ANSI output for render() callback |
| `src/action.rs` | Execute copy/open/paste via RunCommands |
| `src/pane_capture.rs` | Capture target pane content via dump-screen |
| `src/ansi.rs` | Parse tmux-style format strings to ANSI escape codes |
| `src/state.rs` | PluginPhase enum, Modifier enum |

## Patterns

Built-in patterns are defined in `src/config.rs` as `builtin_patterns()`. User patterns are configured via KDL config (`pattern_0`, `pattern_1`, ...). Patterns use Rust `regex` crate syntax. Named capture group `(?P<match>...)` controls which part of a match gets highlighted. When patterns are combined, `match` groups are automatically renamed to `match_0`, `match_1`, etc. to avoid conflicts.

## Testing

Tests use Rust's built-in `#[cfg(test)]` framework with `#[test]` functions. Tests are inline in each module file.

## Version Control

This repository uses **Jujutsu (jj)**, not git, for version control operations.
