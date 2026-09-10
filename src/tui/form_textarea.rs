//! FormEdit textarea layout helpers.
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;

use super::Rect;
use super::editform;

pub(super) const TEXTAREA_MAX_DISPLAY_ROWS: u16 = 12;

pub(super) fn focused_field_virtual_rows(state: &editform::EditFormState) -> (u16, u16) {
    let mut y: u16 = 0;
    for (idx, field) in state.form.fields.iter().enumerate() {
        if !state.field_visible(field) {
            continue;
        }
        let content_rows: u16 = match &field.kind {
            editform::FieldKind::Textarea { rows, .. } => textarea_display_rows(
                state.get(field.id),
                (*rows).max(1),
                None,
                TEXTAREA_MAX_DISPLAY_ROWS,
            ),
            editform::FieldKind::SubForm { .. } => {
                let items_len = state.sub_state.get(field.id).map(|v| v.len()).unwrap_or(0);
                (1 + items_len.max(1)) as u16
            }
            _ => 1,
        };
        let box_height = content_rows.saturating_add(2);
        let entry_height = 1u16.saturating_add(box_height).saturating_add(1);
        if idx == state.focused_field {
            return (y, y.saturating_add(1).saturating_add(box_height));
        }
        y = y.saturating_add(entry_height);
    }
    (0, 0)
}

pub(super) fn textarea_display_rows(
    value: &str,
    base_rows: u16,
    wrap_width: Option<u16>,
    max_rows: u16,
) -> u16 {
    let content_rows = textarea_visual_line_count(value, wrap_width).min(u16::MAX as usize) as u16;
    base_rows.max(content_rows.max(1)).min(max_rows.max(1))
}

pub(super) fn textarea_max_rows_for_window(content_height: u16) -> u16 {
    content_height
        .saturating_sub(3)
        .max(1)
        .min(TEXTAREA_MAX_DISPLAY_ROWS)
}

pub(super) fn textarea_visual_line_count(value: &str, wrap_width: Option<u16>) -> usize {
    visual_lines(value, wrap_width).len().max(1)
}

/// One on-screen row after wrapping at `wrap_width` columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct VisualLine {
    pub start: usize,
    pub len: usize,
}

pub(super) fn visual_lines(value: &str, wrap_width: Option<u16>) -> Vec<VisualLine> {
    let chars: Vec<char> = value.chars().collect();
    let width = wrap_width.map(|w| w.max(1) as usize);
    if chars.is_empty() {
        return vec![VisualLine { start: 0, len: 0 }];
    }
    let mut lines = Vec::new();
    let mut i = 0;
    let mut row_start = 0;
    while i < chars.len() {
        if chars[i] == '\n' {
            lines.push(VisualLine {
                start: row_start,
                len: i - row_start,
            });
            i += 1;
            row_start = i;
            continue;
        }
        if let Some(w) = width {
            if i - row_start == w {
                lines.push(VisualLine {
                    start: row_start,
                    len: w,
                });
                row_start = i;
                continue;
            }
        }
        i += 1;
    }
    lines.push(VisualLine {
        start: row_start,
        len: chars.len() - row_start,
    });
    lines
}

fn visual_row_for_cursor(lines: &[VisualLine], cursor_pos: usize) -> usize {
    lines
        .iter()
        .rposition(|line| line.start <= cursor_pos)
        .unwrap_or(0)
}

/// Wrap width and whether a scrollbar column is reserved for `rect`.
pub(super) fn textarea_wrap_for_rect(value: &str, rect: Rect) -> (u16, bool) {
    let visible = rect.height.max(1) as usize;
    let full = rect.width.max(1);
    let needs_scroll = visual_lines(value, Some(full)).len() > visible && rect.width > 1;
    if needs_scroll {
        (full.saturating_sub(1).max(1), true)
    } else {
        (full, false)
    }
}

pub(super) fn render_textarea_display_window(
    value: &str,
    cursor_pos: usize,
    focused: bool,
    visible_rows: usize,
    wrap_width: Option<u16>,
) -> (String, usize, usize) {
    let visible_rows = visible_rows.max(1);
    let lines = visual_lines(value, wrap_width);
    let chars: Vec<char> = value.chars().collect();
    let cursor_row = visual_row_for_cursor(&lines, cursor_pos).min(lines.len().saturating_sub(1));
    let start = if focused {
        cursor_row.saturating_sub(visible_rows.saturating_sub(1))
    } else {
        0
    };
    let end = (start + visible_rows).min(lines.len());
    let mut display = Vec::with_capacity(visible_rows);
    for line in lines.iter().take(end).skip(start) {
        display.push(chars.iter().skip(line.start).take(line.len).collect());
    }
    while display.len() < visible_rows {
        display.push(String::new());
    }
    (display.join("\n"), start, lines.len())
}

pub(super) fn render_textarea_scrollbar(
    frame: &mut Frame,
    area: Rect,
    first_visible_row: usize,
    visible_rows: usize,
    total_rows: usize,
    scrollbar_color: Color,
    background: Color,
) {
    if area.height == 0 || total_rows <= visible_rows {
        return;
    }
    for y in 0..area.height {
        frame.render_widget(
            Paragraph::new(" ").style(Style::default().bg(background)),
            Rect {
                x: area.x,
                y: area.y + y,
                width: 1,
                height: 1,
            },
        );
    }
    let track_height = area.height as usize;
    let thumb_height = ((visible_rows.max(1) * track_height) / total_rows.max(1))
        .max(1)
        .min(track_height);
    let max_scroll = total_rows.saturating_sub(visible_rows.max(1));
    let travel = track_height.saturating_sub(thumb_height);
    let thumb_top = if max_scroll == 0 {
        0
    } else {
        (first_visible_row.min(max_scroll) * travel) / max_scroll
    };
    for y in thumb_top..thumb_top + thumb_height {
        frame.render_widget(
            Paragraph::new("█").style(Style::default().fg(scrollbar_color).bg(background)),
            Rect {
                x: area.x,
                y: area.y + y as u16,
                width: 1,
                height: 1,
            },
        );
    }
}

pub(super) fn textarea_cursor_row(
    value: &str,
    cursor_pos: usize,
    wrap_width: Option<u16>,
) -> usize {
    let lines = visual_lines(value, wrap_width);
    visual_row_for_cursor(&lines, cursor_pos).min(lines.len().saturating_sub(1))
}

pub(super) fn textarea_cursor_col(
    value: &str,
    cursor_pos: usize,
    wrap_width: Option<u16>,
) -> usize {
    let lines = visual_lines(value, wrap_width);
    let row = visual_row_for_cursor(&lines, cursor_pos);
    cursor_pos.saturating_sub(lines[row].start)
}

pub(super) fn textarea_move_cursor_vertical(
    value: &str,
    cursor_pos: usize,
    row_delta: isize,
    wrap_width: Option<u16>,
) -> usize {
    let lines = visual_lines(value, wrap_width);
    let row = visual_row_for_cursor(&lines, cursor_pos).min(lines.len().saturating_sub(1));
    let col = cursor_pos.saturating_sub(lines[row].start);
    let target_row = row
        .saturating_add_signed(row_delta)
        .min(lines.len().saturating_sub(1));
    let line = lines[target_row];
    line.start + col.min(line.len)
}

pub(super) fn textarea_home(value: &str, cursor_pos: usize, wrap_width: Option<u16>) -> usize {
    let lines = visual_lines(value, wrap_width);
    let row = visual_row_for_cursor(&lines, cursor_pos);
    lines[row].start
}

pub(super) fn textarea_end(value: &str, cursor_pos: usize, wrap_width: Option<u16>) -> usize {
    let lines = visual_lines(value, wrap_width);
    let row = visual_row_for_cursor(&lines, cursor_pos);
    let line = lines[row];
    if line.len == 0 {
        return line.start;
    }
    let soft = row + 1 < lines.len() && lines[row + 1].start == line.start + line.len;
    if soft {
        line.start + line.len - 1
    } else {
        line.start + line.len
    }
}

/// Map a click in the visible text rect to a char cursor index.
pub(super) fn textarea_cursor_from_click(
    value: &str,
    wrap_width: Option<u16>,
    first_visible_row: usize,
    local_x: u16,
    local_y: u16,
) -> usize {
    let lines = visual_lines(value, wrap_width);
    let row = first_visible_row
        .saturating_add(local_y as usize)
        .min(lines.len().saturating_sub(1));
    let line = lines[row];
    line.start + (local_x as usize).min(line.len)
}

#[derive(Clone, Copy, Debug)]
pub(super) struct TextareaHit {
    pub rect: Rect,
    pub field_idx: usize,
    pub first_visible_row: usize,
    pub wrap_width: u16,
}

pub(super) fn auto_scroll_for_focus(state: &editform::EditFormState, current_scroll: u16) -> u16 {
    const ESTIMATED_VISIBLE: u16 = 16;
    let (top, bottom) = focused_field_virtual_rows(state);
    if top < current_scroll {
        top
    } else if bottom > current_scroll.saturating_add(ESTIMATED_VISIBLE) {
        bottom.saturating_sub(ESTIMATED_VISIBLE)
    } else {
        current_scroll
    }
}

pub(super) fn insert_char(value: &str, cursor_pos: usize, ch: char) -> (String, usize) {
    let mut chars: Vec<char> = value.chars().collect();
    let pos = cursor_pos.min(chars.len());
    chars.insert(pos, ch);
    (chars.into_iter().collect(), pos + 1)
}

pub(super) fn delete_char_before(value: &str, cursor_pos: usize) -> (String, usize) {
    let mut chars: Vec<char> = value.chars().collect();
    let pos = cursor_pos.min(chars.len());
    if pos == 0 {
        return (value.to_string(), 0);
    }
    chars.remove(pos - 1);
    (chars.into_iter().collect(), pos - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_splits_long_line() {
        let lines = visual_lines("abcdefghij", Some(4));
        assert_eq!(
            lines,
            vec![
                VisualLine { start: 0, len: 4 },
                VisualLine { start: 4, len: 4 },
                VisualLine { start: 8, len: 2 },
            ]
        );
        assert_eq!(visual_lines("abcd", Some(4)).len(), 1);
        assert_eq!(visual_lines("ab\ncd", Some(4)).len(), 2);
        assert_eq!(visual_lines("ab\n", None).len(), 2);
    }

    #[test]
    fn home_end_logical_line() {
        let v = "hello\nworld";
        assert_eq!(textarea_home(v, 8, None), 6);
        assert_eq!(textarea_end(v, 8, None), 11);
        assert_eq!(textarea_home(v, 2, None), 0);
        assert_eq!(textarea_end(v, 2, None), 5);
    }

    #[test]
    fn home_end_wrapped_line() {
        let v = "abcdefghij";
        assert_eq!(textarea_home(v, 5, Some(4)), 4);
        assert_eq!(textarea_end(v, 5, Some(4)), 7);
        assert_eq!(textarea_home(v, 9, Some(4)), 8);
        assert_eq!(textarea_end(v, 9, Some(4)), 10);
    }

    #[test]
    fn click_sets_cursor() {
        let v = "abcdefghij";
        assert_eq!(textarea_cursor_from_click(v, Some(4), 0, 2, 0), 2);
        assert_eq!(textarea_cursor_from_click(v, Some(4), 0, 1, 1), 5);
        assert_eq!(textarea_cursor_from_click(v, Some(4), 0, 9, 2), 10);
    }

    #[test]
    fn vertical_move_follows_wrap() {
        let v = "abcdefghij";
        assert_eq!(textarea_move_cursor_vertical(v, 1, 1, Some(4)), 5);
        assert_eq!(textarea_move_cursor_vertical(v, 5, -1, Some(4)), 1);
        assert_eq!(textarea_move_cursor_vertical("a\nb\nc", 0, 1, None), 2);
    }

    #[test]
    fn display_window_wraps() {
        let (display, first, total) =
            render_textarea_display_window("abcdefghij", 0, true, 2, Some(4));
        assert_eq!(total, 3);
        assert_eq!(first, 0);
        assert_eq!(display, "abcd\nefgh");
    }
}
