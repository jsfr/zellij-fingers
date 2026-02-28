mod action;
mod ansi;
mod config;
mod hinter;
mod huffman;
mod match_formatter;
mod pane_capture;
mod priority_queue;
mod renderer;
mod state;

use std::collections::BTreeMap;

use zellij_tile::prelude::*;
use crate::config::Config;
use crate::hinter::Hinter;
use crate::state::PluginPhase;

struct ZellijFingers {
    phase: PluginPhase,
    config: Config,
    hinter: Option<Hinter>,
    input: String,
    multi_mode: bool,
    selected_hints: Vec<String>,
    multi_matches: Vec<String>,
    pane_content: Vec<String>,
    pane_rows: usize,
    pane_cols: usize,
    target_pane_id: Option<u32>,
}

impl Default for ZellijFingers {
    fn default() -> Self {
        Self {
            phase: PluginPhase::WaitingForPermissions,
            config: Config::default(),
            hinter: None,
            input: String::new(),
            multi_mode: false,
            selected_hints: Vec::new(),
            multi_matches: Vec::new(),
            pane_content: Vec::new(),
            pane_rows: 0,
            pane_cols: 0,
            target_pane_id: None,
        }
    }
}

register_plugin!(ZellijFingers);

impl ZellijPlugin for ZellijFingers {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.config = Config::from_kdl(&configuration);

        request_permission(&[
            PermissionType::RunCommands,
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);

        subscribe(&[
            EventType::Key,
            EventType::PaneUpdate,
            EventType::RunCommandResult,
            EventType::PermissionRequestResult,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match &self.phase {
            PluginPhase::WaitingForPermissions => {
                if let Event::PermissionRequestResult(PermissionStatus::Granted) = event {
                    // Resize floating pane to fill the entire screen
                    let plugin_ids = get_plugin_ids();
                    let pane_id = PaneId::Plugin(plugin_ids.plugin_id);
                    let coords = FloatingPaneCoordinates::new(
                        Some("0".to_string()),
                        Some("0".to_string()),
                        Some("100%".to_string()),
                        Some("100%".to_string()),
                        None,
                    ).unwrap();
                    change_floating_panes_coordinates(vec![(pane_id, coords)]);

                    self.phase = PluginPhase::Capturing;
                    pane_capture::request_pane_capture();
                }
                false
            }
            PluginPhase::Capturing => {
                match event {
                    Event::PaneUpdate(pane_manifest) => {
                        if self.target_pane_id.is_none() {
                            self.target_pane_id =
                                pane_capture::find_target_pane(&pane_manifest);
                            if let Some((rows, cols)) =
                                pane_capture::get_pane_dimensions(&pane_manifest, self.target_pane_id)
                            {
                                self.pane_rows = rows;
                                self.pane_cols = cols;
                            }
                        }
                        // If we already have content, try to start hinting
                        self.try_start_hinting()
                    }
                    Event::RunCommandResult(exit_code, stdout, _stderr, _context) => {
                        if exit_code == Some(0) {
                            let content = String::from_utf8_lossy(&stdout).to_string();
                            self.pane_content = content
                                .lines()
                                .map(|l| l.trim_end().to_string())
                                .collect();
                        }
                        // If we already have pane dimensions, try to start hinting
                        self.try_start_hinting()
                    }
                    _ => false,
                }
            }
            PluginPhase::Hinting => {
                if let Event::Key(key) = event {
                    self.handle_key(key);
                    true
                } else {
                    false
                }
            }
            PluginPhase::Done => false,
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        match self.phase {
            PluginPhase::Hinting => {
                if let Some(ref mut hinter) = self.hinter {
                    let output = renderer::render(
                        hinter,
                        &self.input,
                        &self.selected_hints,
                        rows,
                        cols,
                    );
                    print!("{}", output);
                }
            }
            PluginPhase::WaitingForPermissions => {
                println!("Waiting for permissions...");
            }
            PluginPhase::Capturing => {
                println!(
                    "Capturing pane content... (target_pane_id: {:?}, content_lines: {})",
                    self.target_pane_id,
                    self.pane_content.len()
                );
            }
            PluginPhase::Done => {}
        }
    }
}

impl ZellijFingers {
    /// Try to transition to Hinting once we have both pane content and dimensions.
    fn try_start_hinting(&mut self) -> bool {
        if !self.pane_content.is_empty() && self.pane_cols > 0 {
            let hinter = Hinter::new(
                &self.pane_content,
                self.pane_cols,
                &self.config,
            );
            self.hinter = Some(hinter);
            self.phase = PluginPhase::Hinting;
            true
        } else {
            false
        }
    }

    fn handle_key(&mut self, key: KeyWithModifier) {
        match key.bare_key {
            BareKey::Esc => {
                close_self();
                self.phase = PluginPhase::Done;
            }
            BareKey::Enter if self.multi_mode => {
                let result = self.multi_matches.join(" ");
                if !result.is_empty() {
                    action::execute_action(&self.config, &result);
                }
                close_self();
                self.phase = PluginPhase::Done;
            }
            BareKey::Tab => {
                self.multi_mode = !self.multi_mode;
                if !self.multi_mode {
                    let result = self.multi_matches.join(" ");
                    if !result.is_empty() {
                        action::execute_action(&self.config, &result);
                    }
                    close_self();
                    self.phase = PluginPhase::Done;
                }
            }
            BareKey::Backspace => {
                self.input.pop();
            }
            BareKey::Char(c) => {
                self.input.push(c.to_ascii_lowercase());
                self.try_match();
            }
            _ => {}
        }
    }

    fn try_match(&mut self) {
        if let Some(ref hinter) = self.hinter {
            if let Some(target) = hinter.lookup(&self.input) {
                let text = target.text.clone();
                if self.multi_mode {
                    self.multi_matches.push(text);
                    self.selected_hints.push(self.input.clone());
                    self.input.clear();
                } else {
                    action::execute_action(&self.config, &text);
                    close_self();
                    self.phase = PluginPhase::Done;
                }
            }
        }
    }
}
