use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};

use super::theme::{AppTheme, color_to_hex};

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
            ("F1", "Open/close this help"),
            ("F2", "Open/close theme source + sampled tokens"),
            ("F3", "Validate the open template"),
            ("p", "Preview in browser (mjml -w + loopback wrapper)"),
            (
                "Shift+E",
                "Export template.mjml + template.html next to the JSON",
            ),
            ("s", "Save template.json (+ .backup on manual save)"),
            ("Ctrl+Q", "Quit (confirms if unsaved; bare q does not quit)"),
        ],
        "•",
        h_style,
        k_style,
        div_style,
        width,
    );

    add_section(
        &mut lines,
        "Help / Theme overlays",
        &[
            ("F1 / Esc", "Close help"),
            ("F2 / Esc", "Close theme"),
            ("j/k or arrows", "Scroll"),
            ("g / G", "Jump to top / bottom"),
            ("PageUp / PageDown", "Scroll by a page"),
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
            ("Left / Right", "Cycle enum, or move cursor"),
            ("A / X", "Add / remove collection row (fonts, social)"),
            ("Ctrl+S", "Save (or return from a drilled-in item)"),
            ("Ctrl+P", "Image picker on src / background_url fields"),
            ("Ctrl+E", "Expand focused textarea to a full-size editor"),
            ("Esc", "Close expanded textarea, or cancel edit"),
            ("Click field", "Focus that input"),
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
    let note = "Autosave rewrites template.json 2s after a change when a path is set. Manual s also writes template.json.backup. JSON-LD and CSS may be invalid while typing; F3 / export / preview require them to parse. Insert picker lists only kinds legal for the current selection. Image picker is rooted at images/ and cannot walk above the template folder. Padding is 1-4 values with px or % (e.g. 10px or 10px 20px); bare numbers are saved as px.";
    for chunk in wrap_to_lines(note, width.saturating_sub(2)) {
        lines.push(Line::from(Span::raw(format!("  {}", chunk))));
    }

    Text::from(lines)
}

pub(crate) fn build_theme_text(
    theme: &AppTheme,
    source: &str,
    status: &Option<String>,
    width: usize,
) -> Text<'static> {
    let h_style = Style::default()
        .fg(theme.modal_header)
        .add_modifier(Modifier::BOLD);
    let k_style = Style::default().fg(theme.text_active_focus);
    let div_style = Style::default().fg(theme.text_secondary);

    let mut lines: Vec<Line<'static>> = Vec::new();

    lines.push(Line::from(Span::styled("Theme", h_style)));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  App: ", k_style),
        Span::raw(format!("dd_emailforge v{}", env!("CARGO_PKG_VERSION"))),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Source: ", k_style),
        Span::raw(format!(
            "{}   (./dd_emailforge_theme.yml or ~/.config/ldnddev/)",
            source
        )),
    ]));
    let status_str = status.as_deref().unwrap_or("OK (loaded cleanly)");
    lines.push(Line::from(vec![
        Span::styled("  Status: ", k_style),
        Span::raw(status_str.to_string()),
    ]));
    lines.push(Line::from(""));

    let rule_len = width.saturating_sub(4).clamp(12, 50);
    let rule = "─".repeat(rule_len);
    lines.push(Line::from(Span::styled(format!("  {}", rule), div_style)));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        "Loaded color tokens (sampled)",
        h_style,
    )));
    lines.push(Line::from(""));

    let tokens: Vec<(&str, Color, &str)> = vec![
        ("base_background", theme.base_background, "app_shell base"),
        ("body_background", theme.body_background, "body panes"),
        (
            "modal_background",
            theme.modal_background,
            "modals & toasts",
        ),
        ("text_primary", theme.text_primary, "primary text"),
        ("text_secondary", theme.text_secondary, "muted text"),
        ("text_disabled", theme.text_disabled, "disabled text"),
        ("text_inverse", theme.text_inverse, "inverted text"),
        ("text_labels", theme.text_labels, "labels at rest"),
        ("text_active_focus", theme.text_active_focus, "focus + keys"),
        ("modal_labels", theme.modal_labels, "modal labels"),
        ("modal_header", theme.modal_header, "section titles bold"),
        (
            "selected_background",
            theme.selected_background,
            "selected row",
        ),
        ("border_default", theme.border_default, "idle pane border"),
        ("border_active", theme.border_active, "focused pane border"),
        (
            "input_border_default",
            theme.input_border_default,
            "idle inputs",
        ),
        (
            "input_border_focus",
            theme.input_border_focus,
            "focused inputs",
        ),
        (
            "input_text_default",
            theme.input_text_default,
            "idle input text",
        ),
        (
            "input_text_focus",
            theme.input_text_focus,
            "focused input text",
        ),
        ("cursor", theme.cursor, "input cursor overlay"),
        ("success", theme.success, "success toasts"),
        ("warning", theme.warning, "warning toasts"),
        ("error", theme.error, "error toasts"),
        ("info", theme.info, "info toasts"),
        ("folders", theme.folders, "picker folders"),
        ("files", theme.files, "picker files"),
        ("links", theme.links, "picker links"),
        ("scrollbar", theme.scrollbar, "scrollbars"),
        ("scrollbar_hover", theme.scrollbar_hover, "scrollbar thumb"),
    ];

    for (name, color, role) in tokens {
        let hex = color_to_hex(color);
        let line = format!("  {:<18} {}   ({})", name, hex, role);
        lines.push(Line::from(Span::raw(line)));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(format!("  {}", rule), div_style)));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::raw(
        "  (All colors from self.theme.*. No hardcodes.)",
    )));

    Text::from(lines)
}

pub(crate) fn count_wrapped_lines(text: &Text, _width: usize) -> usize {
    text.lines.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_text_includes_canonical_token_and_source() {
        let theme = AppTheme::default();
        let text = build_theme_text(&theme, "local", &None, 80);
        let joined: String = text
            .lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(joined.contains("base_background"), "{joined}");
        assert!(joined.contains("local"), "{joined}");
        assert!(joined.contains("OK (loaded cleanly)"), "{joined}");
        assert!(!joined.contains("\nbackground "), "{joined}");
    }
}
