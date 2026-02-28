use std::collections::BTreeMap;

use zellij_tile::prelude::*;

const CAPTURE_FILE: &str = "/tmp/zellij-fingers-capture";

/// Read pane content that was pre-dumped by the DumpScreen keybinding action.
/// The keybinding calls `DumpScreen` before launching the plugin, so the file
/// already exists when the plugin starts.
pub fn request_pane_capture() {
    let context = BTreeMap::new();
    run_command(
        &["cat", CAPTURE_FILE],
        context,
    );
}

/// Find the target pane from a PaneManifest: the focused terminal pane
/// in the current tab that is not a plugin pane.
pub fn find_target_pane(manifest: &PaneManifest) -> Option<u32> {
    for panes in manifest.panes.values() {
        for pane in panes {
            if pane.is_focused && !pane.is_plugin {
                return Some(pane.id);
            }
        }
    }
    None
}

/// Get the dimensions (rows, cols) of a specific pane.
pub fn get_pane_dimensions(
    manifest: &PaneManifest,
    target_id: Option<u32>,
) -> Option<(usize, usize)> {
    let target_id = target_id?;
    for panes in manifest.panes.values() {
        for pane in panes {
            if pane.id == target_id {
                return Some((pane.pane_content_rows, pane.pane_columns));
            }
        }
    }
    None
}
