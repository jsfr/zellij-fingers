use std::collections::HashMap;

/// Converts tmux-style format strings (e.g. "fg=green,bold") directly to ANSI SGR escape sequences.
/// No external process calls needed (unlike the Crystal version which used `tput`).
const RESET: &str = "\x1b[0m";

fn color_map() -> HashMap<&'static str, u8> {
    HashMap::from([
        ("black", 0),
        ("red", 1),
        ("green", 2),
        ("yellow", 3),
        ("blue", 4),
        ("magenta", 5),
        ("cyan", 6),
        ("white", 7),
    ])
}

fn style_map() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("bright", "\x1b[1m"),
        ("bold", "\x1b[1m"),
        ("dim", "\x1b[2m"),
        ("italics", "\x1b[3m"),
        ("underscore", "\x1b[4m"),
        ("reverse", "\x1b[7m"),
    ])
}

pub fn parse_style(input: &str) -> String {
    let mut output = String::new();

    for part in input.split([',', ' ']) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        if let Some(code) = parse_single_style(part) {
            output.push_str(&code);
        }
    }

    output
}

fn parse_single_style(style: &str) -> Option<String> {
    if let Some(rest) = style.strip_prefix("fg=") {
        parse_color(rest, false)
    } else if let Some(rest) = style.strip_prefix("bg=") {
        parse_color(rest, true)
    } else {
        parse_attribute(style)
    }
}

fn parse_color(color: &str, is_bg: bool) -> Option<String> {
    if color == "default" {
        return Some(if is_bg {
            "\x1b[49m".to_string()
        } else {
            "\x1b[39m".to_string()
        });
    }

    // Handle colour/color + numeric code (e.g. colour123, color123)
    let color_code = color
        .strip_prefix("colour")
        .or_else(|| color.strip_prefix("color"));

    if let Some(code_str) = color_code {
        if let Ok(code) = code_str.parse::<u8>() {
            let layer = if is_bg { 48 } else { 38 };
            return Some(format!("\x1b[{layer};5;{code}m"));
        }
    }

    // Handle named colors
    let colors = color_map();
    if let Some(&code) = colors.get(color) {
        let base = if is_bg { 40 } else { 30 };
        return Some(format!("\x1b[{}m", base + code));
    }

    None
}

fn parse_attribute(attr: &str) -> Option<String> {
    let (is_remove, name) = if let Some(stripped) = attr.strip_prefix("no") {
        (true, stripped)
    } else {
        (false, attr)
    };

    let styles = style_map();
    if let Some(&code) = styles.get(name) {
        if is_remove {
            Some(RESET.to_string())
        } else {
            Some(code.to_string())
        }
    } else {
        None
    }
}

/// Format a style string from tmux format, stripping #[] wrappers if present.
pub fn format_style(input: &str) -> String {
    let cleaned = input
        .strip_prefix("#[")
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(input);
    parse_style(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fg_and_bg_colors() {
        let result = parse_style("bg=red,fg=yellow,bold");
        assert_eq!(result, "\x1b[41m\x1b[33m\x1b[1m");
    }

    #[test]
    fn parses_256_colour() {
        let result = parse_style("fg=colour123");
        assert_eq!(result, "\x1b[38;5;123m");
    }

    #[test]
    fn parses_default_color() {
        let result = parse_style("fg=default");
        assert_eq!(result, "\x1b[39m");
    }

    #[test]
    fn parses_bold() {
        let result = parse_style("bold");
        assert_eq!(result, "\x1b[1m");
    }

    #[test]
    fn parses_dim() {
        let result = parse_style("dim");
        assert_eq!(result, "\x1b[2m");
    }

    #[test]
    fn parses_italics() {
        let result = parse_style("italics");
        assert_eq!(result, "\x1b[3m");
    }

    #[test]
    fn parses_multiple_styles() {
        let result = parse_style("fg=green,bold");
        assert_eq!(result, "\x1b[32m\x1b[1m");
    }

    #[test]
    fn format_style_strips_tmux_brackets() {
        let result = format_style("#[fg=green,bold]");
        assert_eq!(result, "\x1b[32m\x1b[1m");
    }

    #[test]
    fn format_style_works_without_brackets() {
        let result = format_style("fg=green,bold");
        assert_eq!(result, "\x1b[32m\x1b[1m");
    }

    #[test]
    fn empty_input_returns_empty() {
        let result = parse_style("");
        assert_eq!(result, "");
    }
}
