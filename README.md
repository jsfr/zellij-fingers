# zellij-fingers

A [Zellij](https://zellij.dev/) WASM plugin that highlights pattern matches (URLs, file paths, git SHAs, IPs, etc.) in terminal panes and lets you select them via keyboard hints.

This is a Rust rewrite of [tmux-fingers](https://github.com/Morantron/tmux-fingers) by [@Morantron](https://github.com/Morantron), adapted for the Zellij terminal multiplexer as a native WASM plugin.

## Features

- Highlights common patterns: URLs, file paths, git SHAs, UUIDs, IPs, hex values, Kubernetes resources, git status output, and diff paths
- Huffman-encoded keyboard hints for efficient selection with minimal keystrokes
- Copy to clipboard, open in browser, or run custom actions
- Multi-select mode (press `Tab`) to select multiple matches
- Configurable styles, patterns, keyboard layouts, and actions
- Multiple keyboard layout support: QWERTY, AZERTY, QWERTZ, Dvorak, Colemak (full, homerow, left-hand, right-hand variants)

## Installation

Requires Rust with the `wasm32-wasip1` target:

```bash
rustup target add wasm32-wasip1
```

Build and install:

```bash
# Using just
just install

# Or manually
cargo build --release
mkdir -p ~/.config/zellij/plugins
cp target/wasm32-wasip1/release/zellij-fingers.wasm ~/.config/zellij/plugins/
```

## Usage

Add a keybinding to your Zellij config (`~/.config/zellij/config.kdl`):

```kdl
shared_except "locked" {
    bind "Ctrl f" {
        LaunchOrFocusPlugin "file:~/.config/zellij/plugins/zellij-fingers.wasm" {
            floating true
        }
    }
}
```

When triggered, the plugin overlays the current pane content with highlighted matches. Type the hint characters to select a match. Press `Esc` to cancel.

### Multi-select mode

Press `Tab` to enter multi-select mode. Select multiple matches, then press `Enter` (or `Tab` again) to execute the action on all selected matches joined by spaces.

## Configuration

Pass configuration options as plugin parameters in KDL:

```kdl
LaunchOrFocusPlugin "file:~/.config/zellij/plugins/zellij-fingers.wasm" {
    floating true

    // Action to perform: ":copy:", ":open:", or a custom shell command
    action ":copy:"

    // Hint position relative to the match: "left" or "right"
    hint_position "left"

    // Keyboard layout for hint character ordering
    keyboard_layout "qwerty"

    // Styles use tmux-style format strings
    hint_style "fg=green,bold"
    highlight_style "fg=yellow"
    selected_hint_style "fg=blue,bold"
    selected_highlight_style "fg=blue"
    backdrop_style "dim"

    // Which built-in patterns to enable: "all" or comma-separated names
    // Available: ip, uuid, sha, digit, url, path, hex, kubernetes, git-status, git-status-branch, diff
    enabled_builtin_patterns "all"

    // Custom patterns (Rust regex syntax)
    // Use (?P<match>...) to control which part of the match gets highlighted
    pattern_0 "my-custom-[0-9]+"
    pattern_1 "ERROR: (?P<match>.+)"

    // Override clipboard command (auto-detects pbcopy/wl-copy/xclip/xsel/clip.exe)
    // clipboard_command "pbcopy"

    // Override open command (auto-detects open/xdg-open/cygstart)
    // open_command "open"
}
```

## Credits

This project is a Rust/Zellij rewrite of [tmux-fingers](https://github.com/Morantron/tmux-fingers) by [@Morantron](https://github.com/Morantron), which was originally written in Crystal for tmux. The core concepts -- pattern matching with Huffman-encoded hints -- originate from that project.

## License

MIT
