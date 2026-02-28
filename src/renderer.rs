use crate::hinter::Hinter;

const HIDE_CURSOR: &str = "\x1b[?25l";

/// Renders the hinter output as ANSI text for the plugin's render() callback.
/// Zellij auto-clears between render calls, so we just output the content.
pub fn render(
    hinter: &mut Hinter,
    input_prefix: &str,
    selected_hints: &[String],
    rows: usize,
    cols: usize,
) -> String {
    let lines = hinter.run(input_prefix, selected_hints, cols);
    let mut output = String::new();

    output.push_str(HIDE_CURSOR);

    for (i, line) in lines.iter().enumerate() {
        if i >= rows {
            break;
        }
        output.push_str(&line.content);
        if i < lines.len() - 1 && i < rows - 1 {
            output.push('\n');
        }
    }

    output
}
