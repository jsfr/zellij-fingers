use std::collections::BTreeMap;

use zellij_tile::prelude::*;

use crate::config::Config;

/// Execute the configured action for the matched text.
pub fn execute_action(config: &Config, text: &str) {
    let action = &config.action;

    if action.is_empty() {
        return;
    }

    match action.as_str() {
        ":copy:" => copy_to_clipboard(config, text),
        ":open:" => open_url(config, text),
        _ => run_custom_action(action, text),
    }
}

fn copy_to_clipboard(config: &Config, text: &str) {
    let cmd = if let Some(ref clipboard_cmd) = config.clipboard_command {
        clipboard_cmd.clone()
    } else {
        // Shell fallback chain: try each clipboard tool in order
        concat!(
            "{ pbcopy 2>/dev/null || wl-copy 2>/dev/null || ",
            "xclip -selection clipboard 2>/dev/null || ",
            "xsel -i --clipboard 2>/dev/null || ",
            "clip.exe 2>/dev/null; }"
        )
        .to_string()
    };

    let escaped = shell_escape(text);
    let full_cmd = format!("printf '%s' {} | {}", escaped, cmd);

    let context = BTreeMap::new();
    run_command(&["sh", "-c", &full_cmd], context);
}

fn open_url(config: &Config, text: &str) {
    let context = BTreeMap::new();

    if let Some(ref open_cmd) = config.open_command {
        let escaped = shell_escape(text);
        let full_cmd = format!("{} {}", open_cmd, escaped);
        run_command(&["sh", "-c", &full_cmd], context);
    } else {
        // Try each opener directly without shell wrapper
        // run_command is async so we fire all and let whichever exists succeed
        run_command(&["open", text], context.clone());
        run_command(&["xdg-open", text], context.clone());
        run_command(&["cygstart", text], context);
    }
}

fn run_custom_action(action: &str, text: &str) {
    let escaped = shell_escape(text);
    let full_cmd = format!(
        "HINT={} printf '%s' {} | {}",
        escaped, escaped, action
    );

    let context = BTreeMap::new();
    run_command(&["sh", "-c", &full_cmd], context);
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
