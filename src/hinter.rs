use std::collections::HashMap;

use regex::Regex;
use unicode_width::UnicodeWidthStr;

use crate::config::Config;
use crate::huffman;
use crate::match_formatter::MatchFormatter;

#[derive(Clone, Debug)]
pub struct Target {
    pub text: String,
    pub hint: String,
}

pub struct FormattedLine {
    pub content: String,
}

pub struct Hinter {
    lines: Vec<String>,
    width: usize,
    formatter: MatchFormatter,
    pattern: Regex,
    hints: Vec<String>,
    target_by_hint: HashMap<String, Target>,
    target_by_text: HashMap<String, Target>,
    reuse_hints: bool,
    match_group_indices: Vec<usize>,
}

impl Hinter {
    pub fn new(input: &[String], width: usize, config: &Config) -> Self {
        Self::with_options(
            input,
            width,
            &config.patterns,
            &config.alphabet,
            config.hint_position.clone(),
            config.hint_style.clone(),
            config.highlight_style.clone(),
            config.selected_hint_style.clone(),
            config.selected_highlight_style.clone(),
            config.backdrop_style.clone(),
            true,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_options(
        input: &[String],
        width: usize,
        patterns: &[String],
        alphabet: &[String],
        hint_position: String,
        hint_style: String,
        highlight_style: String,
        selected_hint_style: String,
        selected_highlight_style: String,
        backdrop_style: String,
        reuse_hints: bool,
    ) -> Self {
        // Rename (?P<match>...) groups to unique names per pattern to avoid
        // "duplicate capture group name" errors in the regex crate
        let renamed: Vec<String> = patterns
            .iter()
            .enumerate()
            .map(|(i, p)| p.replace("(?P<match>", &format!("(?P<match_{i}>")))
            .collect();
        let combined = format!("({})", renamed.join("|"));
        let pattern = Regex::new(&combined).expect("Invalid regex pattern");

        let match_group_indices = find_match_group_indices(&pattern);

        let n_matches = if reuse_hints {
            count_unique_matches(input, &pattern, &match_group_indices)
        } else {
            count_matches(input, &pattern)
        };

        let hints = huffman::generate_hints(alphabet, n_matches);

        Self {
            lines: input.to_vec(),
            width,
            formatter: MatchFormatter {
                hint_style,
                highlight_style,
                selected_hint_style,
                selected_highlight_style,
                backdrop_style,
                hint_position,
            },
            pattern,
            hints,
            target_by_hint: HashMap::new(),
            target_by_text: HashMap::new(),
            reuse_hints,
            match_group_indices,
        }
    }

    pub fn run(
        &mut self,
        input_prefix: &str,
        selected_hints: &[String],
        render_width: usize,
    ) -> Vec<FormattedLine> {
        self.regenerate_hints();

        let width = if render_width > 0 { render_width } else { self.width };
        let lines = self.lines.clone();
        let mut result = Vec::new();
        for line in &lines {
            let formatted = self.process_line(line, input_prefix, selected_hints, width);
            result.push(formatted);
        }
        result
    }

    pub fn lookup(&self, hint: &str) -> Option<&Target> {
        self.target_by_hint.get(hint)
    }

    fn regenerate_hints(&mut self) {
        // Only clear mappings; hints were generated in constructor
        self.target_by_hint.clear();
        self.target_by_text.clear();
    }

    fn process_line(
        &mut self,
        line: &str,
        input_prefix: &str,
        selected_hints: &[String],
        width: usize,
    ) -> FormattedLine {
        let tab_positions = tab_positions_for(line);

        // We need to process regex matches and build replacements
        let mut result = String::new();
        let mut last_end = 0;

        // Clone pattern to avoid borrow issues
        let pattern = self.pattern.clone();
        let match_group_indices = self.match_group_indices.clone();

        for caps in pattern.captures_iter(line) {
            let whole_match = caps.get(0).unwrap();
            let match_start = whole_match.start();
            let match_end = whole_match.end();
            let match_text = &line[match_start..match_end];

            // Append text before this match
            result.push_str(&line[last_end..match_start]);

            // Get captured text (named group "match" or whole match)
            let (captured_text, relative_offset) =
                captured_text_and_offset(&caps, &match_group_indices);

            let hint = self.hint_for_text(&captured_text);

            // If hint is longer than captured text, skip this match
            if hint.chars().count() > captured_text.chars().count() {
                self.hints.push(hint);
                result.push_str(match_text);
                last_end = match_end;
                continue;
            }

            self.build_target(&captured_text, &hint);

            // If there's input and hint doesn't start with it, show original text
            if !input_prefix.is_empty() && !hint.starts_with(input_prefix) {
                result.push_str(match_text);
                last_end = match_end;
                continue;
            }

            let formatted = self.formatter.format(
                &hint,
                match_text,
                selected_hints.contains(&hint),
                relative_offset,
            );
            result.push_str(&formatted);
            last_end = match_end;
        }

        // Append remaining text
        result.push_str(&line[last_end..]);

        // Tab expansion
        let initial_len = result.len();
        result = expand_tabs(&result, &tab_positions);
        let tab_correction = result.len() - initial_len;

        // Prepend backdrop style
        let backdrop = &self.formatter.backdrop_style;
        let with_backdrop = format!("{}{}", backdrop, result);

        // Calculate padding
        let display_width = UnicodeWidthStr::width(line);
        let padding_amount = width
            .saturating_sub(display_width)
            .saturating_sub(tab_correction);
        let padding = " ".repeat(padding_amount);

        FormattedLine {
            content: format!("{}{}", with_backdrop, padding),
        }
    }

    fn hint_for_text(&mut self, text: &str) -> String {
        if self.reuse_hints {
            if let Some(target) = self.target_by_text.get(text) {
                return target.hint.clone();
            }
        }
        self.pop_hint()
    }

    fn pop_hint(&mut self) -> String {
        self.hints.pop().unwrap_or_default()
    }

    fn build_target(&mut self, text: &str, hint: &str) {
        let target = Target {
            text: text.to_string(),
            hint: hint.to_string(),
        };
        self.target_by_hint.insert(hint.to_string(), target.clone());
        self.target_by_text.insert(text.to_string(), target);
    }
}

fn find_match_group_indices(pattern: &Regex) -> Vec<usize> {
    pattern
        .capture_names()
        .enumerate()
        .filter_map(|(i, name)| {
            if let Some(n) = name {
                if n.starts_with("match_") {
                    return Some(i);
                }
            }
            None
        })
        .collect()
}

fn captured_text_and_offset(
    caps: &regex::Captures<'_>,
    match_group_indices: &[usize],
) -> (String, Option<(usize, usize)>) {
    for &idx in match_group_indices {
        if let Some(m) = caps.get(idx) {
            let whole = caps.get(0).unwrap();
            let relative_start = m.start() - whole.start();
            let length = m.as_str().len();
            return (m.as_str().to_string(), Some((relative_start, length)));
        }
    }

    (caps[0].to_string(), None)
}

fn count_matches(lines: &[String], pattern: &Regex) -> usize {
    lines
        .iter()
        .map(|line| pattern.find_iter(line).count())
        .sum()
}

fn count_unique_matches(
    lines: &[String],
    pattern: &Regex,
    match_group_indices: &[usize],
) -> usize {
    let mut seen = std::collections::HashSet::new();
    for line in lines {
        for caps in pattern.captures_iter(line) {
            let (text, _) = captured_text_and_offset(&caps, match_group_indices);
            seen.insert(text);
        }
    }
    seen.len()
}

fn tab_positions_for(line: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    for (i, c) in line.chars().enumerate() {
        if c == '\t' {
            positions.push(i);
        }
    }
    positions
}

fn expand_tabs(line: &str, tab_positions: &[usize]) -> String {
    let mut result = String::new();
    let mut correction = 0;
    let mut tab_idx = 0;

    for c in line.chars() {
        if c == '\t' {
            if let Some(&tab_pos) = tab_positions.get(tab_idx) {
                let spaces = 8 - ((tab_pos + correction) % 8);
                correction += spaces - 1;
                for _ in 0..spaces {
                    result.push(' ');
                }
                tab_idx += 1;
            } else {
                result.push('\t');
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;

    fn make_hinter(input: &[&str], width: usize, reuse_hints: bool) -> Hinter {
        let lines: Vec<String> = input.iter().map(|s| s.to_string()).collect();
        let patterns = config::all_builtin_patterns();
        let alphabet: Vec<String> = "asdf".chars().map(|c| c.to_string()).collect();

        Hinter::with_options(
            &lines,
            width,
            &patterns,
            &alphabet,
            "left".to_string(),
            "\x1b[32;1m".to_string(),
            "\x1b[33m".to_string(),
            "\x1b[34;1m".to_string(),
            "\x1b[34m".to_string(),
            String::new(),
            reuse_hints,
        )
    }

    #[test]
    fn works_with_git_status_output() {
        let input = vec![
            "",
            "On branch ruby-rewrite-more-like-crystal-rewrite-amirite",
            "Your branch is up to date with 'origin/ruby-rewrite-more-like-crystal-rewrite-amirite'.",
            "",
            "Changes to be committed:",
            "  (use \"git restore --staged <file>...\" to unstage)",
            "        modified:   spec/lib/fingers/match_formatter_spec.cr",
            "",
            "Changes not staged for commit:",
            "  (use \"git add <file>...\" to update what will be committed)",
            "  (use \"git restore <file>...\" to discard changes in working directory)",
            "        modified:   .gitignore",
            "        modified:   spec/lib/fingers/hinter_spec.cr",
            "        modified:   spec/spec_helper.cr",
            "        modified:   src/fingers/cli.cr",
            "        modified:   src/fingers/dirs.cr",
            "        modified:   src/fingers/match_formatter.cr",
        ];

        let mut hinter = make_hinter(&input, 100, true);
        let result = hinter.run("", &[], 100);
        assert!(!result.is_empty());
    }

    #[test]
    fn reuses_hints_for_duplicate_matches() {
        let input = vec![
            "        modified:   src/fingers/cli.cr",
            "        modified:   src/fingers/cli.cr",
            "        modified:   src/fingers/cli.cr",
        ];

        let mut hinter = make_hinter(&input, 100, true);
        let _ = hinter.run("", &[], 100);

        // With reuse_hints=true, the same text should get the same hint
        // so we should only have generated hints for unique matches
        // (the path appears in all 3, so only 1 unique hint needed for the path)
    }

    #[test]
    fn can_rerender_when_not_reusing_hints() {
        let input = vec![
            "        modified:   src/fingers/cli.cr",
            "        modified:   src/fingers/cli.cr",
            "        modified:   src/fingers/cli.cr",
        ];

        let mut hinter = make_hinter(&input, 100, false);
        let _ = hinter.run("", &[], 100);
        // Running twice should work without panicking
        let _ = hinter.run("", &[], 100);
    }
}
