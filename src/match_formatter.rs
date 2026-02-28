//! Formats a match with its hint overlaid, using ANSI escape codes.
//!
//! Handles hint positioning (left/right), selected/unselected states,
//! and offset-based partial highlighting (for named capture groups).

const RESET: &str = "\x1b[0m";

pub struct MatchFormatter {
    pub hint_style: String,
    pub highlight_style: String,
    pub selected_hint_style: String,
    pub selected_highlight_style: String,
    pub backdrop_style: String,
    pub hint_position: String,
}

impl MatchFormatter {
    pub fn format(
        &self,
        hint: &str,
        highlight: &str,
        selected: bool,
        offset: Option<(usize, usize)>,
    ) -> String {
        let mut result = String::new();
        result.push_str(RESET);
        result.push_str(&self.before_offset(offset, highlight));
        result.push_str(&self.format_offset(selected, hint, &self.within_offset(offset, highlight)));
        result.push_str(&self.after_offset(offset, highlight));
        result.push_str(&self.backdrop_style);
        result
    }

    fn before_offset(&self, offset: Option<(usize, usize)>, highlight: &str) -> String {
        match offset {
            None => String::new(),
            Some((start, _)) => {
                let prefix: String = highlight.chars().take(start).collect();
                format!("{}{}", self.backdrop_style, prefix)
            }
        }
    }

    fn within_offset(&self, offset: Option<(usize, usize)>, highlight: &str) -> String {
        match offset {
            None => highlight.to_string(),
            Some((start, length)) => {
                highlight.chars().skip(start).take(length).collect()
            }
        }
    }

    fn after_offset(&self, offset: Option<(usize, usize)>, highlight: &str) -> String {
        match offset {
            None => String::new(),
            Some((start, length)) => {
                let suffix: String = highlight.chars().skip(start + length).collect();
                format!("{}{}", self.backdrop_style, suffix)
            }
        }
    }

    fn format_offset(&self, selected: bool, hint: &str, highlight: &str) -> String {
        let chopped = self.chop_highlight(hint, highlight);

        let hint_style = if selected {
            &self.selected_hint_style
        } else {
            &self.hint_style
        };
        let highlight_style = if selected {
            &self.selected_highlight_style
        } else {
            &self.highlight_style
        };

        let hint_pair = format!("{}{}{}", hint_style, hint, RESET);
        let highlight_pair = format!("{}{}{}", highlight_style, chopped, RESET);

        if self.hint_position == "right" {
            format!("{}{}", highlight_pair, hint_pair)
        } else {
            format!("{}{}", hint_pair, highlight_pair)
        }
    }

    fn chop_highlight(&self, hint: &str, highlight: &str) -> String {
        let hint_len = hint.chars().count();
        let highlight_chars: Vec<char> = highlight.chars().collect();

        if highlight_chars.len() <= hint_len {
            return String::new();
        }

        if self.hint_position == "right" {
            highlight_chars[..highlight_chars.len() - hint_len]
                .iter()
                .collect()
        } else {
            highlight_chars[hint_len..].iter().collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup(
        hint_position: &str,
        selected: bool,
        offset: Option<(usize, usize)>,
        hint: &str,
        highlight: &str,
    ) -> String {
        let formatter = MatchFormatter {
            hint_style: "#[fg=yellow,bold]".to_string(),
            highlight_style: "#[fg=yellow]".to_string(),
            selected_hint_style: "#[fg=green,bold]".to_string(),
            selected_highlight_style: "#[fg=green]".to_string(),
            backdrop_style: "#[bg=black,fg=white]".to_string(),
            hint_position: hint_position.to_string(),
        };

        formatter.format(hint, highlight, selected, offset)
    }

    #[test]
    fn hint_position_left() {
        let result = setup("left", false, None, "a", "yolo");
        assert_eq!(
            result,
            "\x1b[0m#[fg=yellow,bold]a\x1b[0m#[fg=yellow]olo\x1b[0m#[bg=black,fg=white]"
        );
    }

    #[test]
    fn hint_position_right() {
        let result = setup("right", false, None, "a", "yolo");
        assert_eq!(
            result,
            "\x1b[0m#[fg=yellow]yol\x1b[0m#[fg=yellow,bold]a\x1b[0m#[bg=black,fg=white]"
        );
    }

    #[test]
    fn selected_hint() {
        let result = setup("left", true, None, "a", "yolo");
        assert_eq!(
            result,
            "\x1b[0m#[fg=green,bold]a\x1b[0m#[fg=green]olo\x1b[0m#[bg=black,fg=white]"
        );
    }

    #[test]
    fn with_offset() {
        let result = setup("left", false, Some((1, 5)), "a", "yoloyoloyolo");
        assert_eq!(
            result,
            "\x1b[0m#[bg=black,fg=white]y#[fg=yellow,bold]a\x1b[0m#[fg=yellow]loyo\x1b[0m#[bg=black,fg=white]loyolo#[bg=black,fg=white]"
        );
    }
}
