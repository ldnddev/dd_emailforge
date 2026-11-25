use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};

use super::theme::AppTheme;

fn wrap_to_lines(text: &str, width: usize) -> Vec<String> {
    let w = width.max(1);
    if text.is_empty() {
        return vec![String::new()];
    }
    let mut out: Vec<String> = vec![];
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
            if current.chars().count() > w {
                let chars: Vec<char> = current.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    let end = (i + w).min(chars.len());
                    out.push(chars[i..end].iter().collect());
                    i = end;
                }
                current.clear();
            }
            continue;
        }
        let with_space = format!("{} {}", current, word);
        if with_space.chars().count() <= w {
            current = with_space;
        } else {
            if !current.is_empty() {
                out.push(current);
            }
            current = word.to_string();
            if current.chars().count() > w {
                let chars: Vec<char> = current.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    let end = (i + w).min(chars.len());
                    out.push(chars[i..end].iter().collect());
                    i = end;
                }
                current.clear();
            }
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

pub(crate) fn build_help_text(theme: &AppTheme, width: usize) -> Text<'static> {
    let h_style = Style::default()
        .fg(theme.modal_header)
        .add_modifier(Modifier::BOLD);
    let k_style = Style::default().fg(theme.text_active_focus);
    let div_style = Style::default().fg(theme.text_secondary);

    const KEY_COL: usize = 22;

    fn add_section(
        lines: &mut Vec<Line<'static>>,
        title: &'static str,
        items: &[(&'static str, &'static str)],
        icon: &str,
        h_style: Style,
        k_style: Style,
        div_style: Style,
        width: usize,
    ) {
        lines.push(Line::from(Span::styled(title.to_string(), h_style)));
        lines.push(Line::from(""));
        for (k, a) in items {
            let prefix = format!("  {} {:<18}", icon, k);
            let avail = width.saturating_sub(KEY_COL);
            let chunks = wrap_to_lines(a, avail);
            if chunks.is_empty() || (chunks.len() == 1 && chunks[0].is_empty()) {
                lines.push(Line::from(Span::styled(prefix, k_style)));
            } else {
                for (i, chunk) in chunks.iter().enumerate() {
                    if i == 0 {
                        lines.push(Line::from(vec![
                            Span::styled(prefix.clone(), k_style),
                            Span::raw(chunk.clone()),
                        ]));
                    } else {
                        let cont = format!("{}{}", " ".repeat(KEY_COL), chunk);
                        lines.push(Line::from(Span::raw(cont)));
                    }
                }
            }
        }
        lines.push(Line::from(""));
        let rule_len = width.saturating_sub(4).clamp(12, 50);
        let rule = "─".repeat(rule_len);
        lines.push(Line::from(Span::styled(format!("  {}", rule), div_style)));
        lines.push(Line::from(""));
    }

    let mut lines: Vec<Line<'static>> = Vec::new();

    add_section(
        &mut lines,
        "Global",
        &[
            ("F1", "Open/close Settings (Themes, then this Help)"),
            ("Ctrl+Q", "Quit (confirms if unsaved; bare q does not quit)"),
            ("F2", "Same toggle as F1"),
            ("F3", "Validate; Enter on an error jumps to the node/field"),
            ("n", "New template (starter + folder)"),
            ("o", "Open template.json or a folder"),
            ("p", "Preview in browser (mjml -w + loopback wrapper)"),
            (
                "Shift+E",
                "Export template.mjml + template.html next to the JSON",
            ),
            ("s", "Save template.json (+ .backup on manual save)"),
        ],
        "•",
        h_style,
        k_style,
        div_style,
        width,
    );

    add_section(
        &mut lines,
        "Settings",
        &[
            ("F1 / F2", "Close Settings and revert unsaved theme edits"),
            ("Esc", "Close Help, or step back from the color editor"),
            ("Tab / [ / ]", "Change page"),
            ("j/k or arrows", "Move the theme row. On Help, this scrolls"),
            (
                "Enter",
                "Apply the highlighted theme, or edit a color's hex",
            ),
            ("e", "Edit colors"),
            ("y / s", "Save the theme"),
            ("t", "Toggle save target local / global"),
            (
                "PageUp / PageDown",
                "Scroll this page without moving the row",
            ),
            ("g / G", "Jump to top / bottom"),
        ],
        "•",
        h_style,
        k_style,
        div_style,
        width,
    );

    add_section(
        &mut lines,
        "Structure tree",
        &[
            ("j/k or arrows", "Move selection"),
            ("g / G", "First / last row"),
            ("h / l", "Collapse / expand"),
            ("Space", "Toggle expand"),
            ("Enter", "Edit selected (FormEdit)"),
            ("/", "Insert component (legal kinds only)"),
            ("Tab", "Focus Structure ↔ Details"),
            ("d", "Delete (not HEAD/BRAND/BODY)"),
            ("y", "Duplicate after"),
            ("u", "Undo (cap 20)"),
            ("U / Ctrl+R", "Redo"),
            ("J / K", "Reorder sibling down / up"),
            ("C / V", "Add / remove column"),
            ("c / v", "Prev / next column"),
        ],
        "•",
        h_style,
        k_style,
        div_style,
        width,
    );

    add_section(
        &mut lines,
        "FormEdit",
        &[
            ("Tab / Shift+Tab", "Next / previous field"),
            ("Up / Down", "Previous / next field (textarea: move line)"),
            (
                "Left / Right",
                "Cycle enum, move cursor, or move among checkboxes",
            ),
            (
                "Home / End",
                "Start / end of textarea line (Ctrl: document)",
            ),
            ("Space", "Toggle focused checkbox (border sides)"),
            ("A / X", "Add / remove collection row (fonts, social)"),
            ("Ctrl+S", "Save (rewrites template.mjml for live preview)"),
            ("Ctrl+P", "Image picker on src / background_url fields"),
            ("Ctrl+E", "Expand focused textarea to a full-size editor"),
            ("Esc", "Close expanded textarea, or cancel edit"),
            ("Paste", "Insert clipboard text (newlines only in textarea)"),
            ("Click field", "Focus that input"),
            ("Click textarea", "Place the cursor"),
        ],
        "•",
        h_style,
        k_style,
        div_style,
        width,
    );

    add_section(
        &mut lines,
        "Mouse",
        &[
            ("Wheel", "Scroll the pane under the cursor"),
            ("Click tree row", "Select (glyph column expands)"),
            ("Click pane", "Focus Structure or Details"),
            ("Click blueprint", "Select that element in the tree"),
            ("Double-click", "Same as Enter (open FormEdit)"),
        ],
        "•",
        h_style,
        k_style,
        div_style,
        width,
    );

    lines.push(Line::from(Span::styled("Notes", h_style)));
    lines.push(Line::from(""));
    let note = "Bare dd_emailforge (or a path) opens the TUI. n creates a starter folder; o opens template.json or a folder. Autosave rewrites template.json 2s after a change when a path is set. Manual s also writes template.json.backup. JSON-LD and CSS may be invalid while typing; F3 / export / preview require them to parse. Enter on an F3 error jumps to that node and focuses the field. Insert picker lists only kinds legal for the current selection. Image picker is rooted at images/ and cannot walk above the template folder. Padding is 1-4 values with px or % (e.g. 10px or 10px 20px); bare numbers are saved as px. Components that take a CSS border also have Border sides checkboxes (All / Top / Right / Bottom / Left); All is the default and omitted from JSON. Empty color, font-family, and button color fields inherit brand; FormEdit shows the live brand value as a placeholder.";
    for chunk in wrap_to_lines(note, width.saturating_sub(2)) {
        lines.push(Line::from(Span::raw(format!("  {}", chunk))));
    }

    Text::from(lines)
}
