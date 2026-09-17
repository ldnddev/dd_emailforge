//! Frame layout: header, empty body, footer, F1/F2 overlays, toasts.
use super::details::{DetailHit, readable_fg};
use super::help::{build_help_text, build_theme_text, count_wrapped_lines};
use super::tree::TreeId;
use super::*;

impl App {
    pub(super) fn draw(&mut self, frame: &mut ratatui::Frame) {
        self.prune_toasts();

        frame.render_widget(Block::default().style(self.theme.app_shell), frame.area());

        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(frame.area());

        let header_block = Block::default()
            .title("dd_emailforge")
            .borders(Borders::ALL)
            .border_style(self.theme.active_border)
            .style(self.theme.app_shell)
            .title_style(
                Style::default()
                    .fg(self.theme.text_active_focus)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_widget(header_block.clone(), root[0]);

        if root[0].height >= 3 {
            let inner = header_block.inner(root[0]);
            let quote = Paragraph::new(self.header_copy.as_str()).style(
                Style::default()
                    .fg(self.theme.text_primary)
                    .bg(self.theme.base_background),
            );
            frame.render_widget(quote, inner);
        }

        self.body_area = root[1];
        self.render_body(frame, root[1]);

        let footer_text = self.footer_hint(root[2].width);
        let footer = Paragraph::new(footer_text).style(self.theme.app_shell);
        frame.render_widget(footer, root[2]);

        self.render_modal(frame);

        if self.show_help {
            self.render_scroll_modal(
                frame,
                "Key & Mouse bindings (F1 / Esc to close, j/k or arrows to scroll)",
                true,
            );
        }

        if self.show_theme {
            if let Some(editor) = &self.theme_editor {
                self.render_theme_editor(frame, editor);
            } else {
                self.render_scroll_modal(
                    frame,
                    "Theme (F2 / Esc to close, j/k or arrows to scroll)",
                    false,
                );
            }
        }

        self.render_toasts(frame, frame.area());
    }

    fn render_body(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        use super::details::{details_title, details_view};
        use super::tree::{PaneFocus, master_detail_tree_width, selected_label};

        self.clamp_tree_selection();
        let rows = self.tree_rows();
        let tree_w = master_detail_tree_width(area.width);
        self.details_visible = tree_w.is_some();
        if !self.details_visible && self.pane == PaneFocus::Details {
            self.pane = PaneFocus::Structure;
        }

        let (tree_area, details_area) = if let Some(tw) = tree_w {
            let split = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(tw), Constraint::Min(0)])
                .split(area);
            (split[0], split[1])
        } else {
            (area, Rect::default())
        };
        self.tree_area = tree_area;
        self.details_area = details_area;

        let tree_active = self.pane == PaneFocus::Structure;
        let tree_border = if tree_active {
            self.theme.border_active
        } else {
            self.theme.border_default
        };
        let tree_block = Block::default()
            .title("Structure")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(tree_border))
            .style(
                Style::default()
                    .fg(self.theme.text_primary)
                    .bg(self.theme.body_background),
            );
        let tree_inner = tree_block.inner(tree_area);
        frame.render_widget(tree_block, tree_area);

        let visible = tree_inner.height as usize;
        if self.selected_row < self.tree_scroll {
            self.tree_scroll = self.selected_row;
        }
        if visible > 0 && self.selected_row >= self.tree_scroll + visible {
            self.tree_scroll = self.selected_row + 1 - visible;
        }

        let lines: Vec<Line> = rows
            .iter()
            .enumerate()
            .map(|(i, row)| {
                let text = format!("{}{}", row.prefix, row.label);
                let style = if i == self.selected_row {
                    Style::default()
                        .fg(self.theme.text_active_focus)
                        .bg(self.theme.selected_background)
                } else {
                    Style::default().fg(self.theme.text_primary)
                };
                Line::from(Span::styled(text, style))
            })
            .collect();
        frame.render_widget(
            Paragraph::new(lines)
                .style(
                    Style::default()
                        .fg(self.theme.text_primary)
                        .bg(self.theme.body_background),
                )
                .scroll((self.tree_scroll as u16, 0)),
            tree_inner,
        );
        let max_tree_scroll = rows.len().saturating_sub(visible);
        if self.tree_scroll > max_tree_scroll {
            self.tree_scroll = max_tree_scroll;
        }
        if rows.len() > visible && visible > 0 {
            paint_scrollbar(
                frame,
                tree_inner,
                self.tree_scroll,
                rows.len(),
                self.theme.scrollbar,
                self.theme.scrollbar_hover,
                self.theme.body_background,
            );
        }

        if details_area.width == 0 {
            return;
        }
        let details_active = self.pane == PaneFocus::Details;
        let details_border = if details_active {
            self.theme.border_active
        } else {
            self.theme.border_default
        };
        let label = selected_label(&rows, self.selected_row);
        let details_block = Block::default()
            .title(details_title(&label))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(details_border))
            .style(
                Style::default()
                    .fg(self.theme.text_primary)
                    .bg(self.theme.body_background),
            );
        let details_inner = details_block.inner(details_area);
        frame.render_widget(details_block, details_area);

        let (text_lines, hits) = details_view(
            self.template.as_ref(),
            rows.get(self.selected_row),
            details_inner.width as usize,
        );
        let selected_id = rows.get(self.selected_row).map(|r| r.id.clone());
        self.details_scroll_max = text_lines
            .len()
            .saturating_sub(details_inner.height as usize);
        if selected_id != self.details_sync_id {
            self.details_sync_id = selected_id.clone();
            if let Some(id) = &selected_id {
                if let Some(hit) = hits.iter().find(|h| &h.id == id) {
                    let vis = details_inner.height as usize;
                    if vis > 0 {
                        if hit.line < self.details_scroll {
                            self.details_scroll = hit.line;
                        } else if hit.line >= self.details_scroll + vis {
                            self.details_scroll = hit.line.saturating_sub(vis.saturating_sub(1));
                        }
                    }
                }
            }
        }
        if self.details_scroll > self.details_scroll_max {
            self.details_scroll = self.details_scroll_max;
        }
        self.details_hit_areas.clear();
        for hit in &hits {
            if hit.line < self.details_scroll {
                continue;
            }
            let screen_y = hit.line - self.details_scroll;
            if screen_y >= details_inner.height as usize {
                continue;
            }
            let x0 = hit.x0.min(details_inner.width as usize) as u16;
            let x1 = hit.x1.min(details_inner.width as usize) as u16;
            if x1 <= x0 {
                continue;
            }
            self.details_hit_areas.push((
                Rect {
                    x: details_inner.x + x0,
                    y: details_inner.y + screen_y as u16,
                    width: x1.saturating_sub(x0).max(1),
                    height: 1,
                },
                hit.id.clone(),
            ));
        }
        let para_lines: Vec<Line> = text_lines
            .iter()
            .enumerate()
            .map(|(i, l)| style_blueprint_line(l, i, &hits, selected_id.as_ref(), &self.theme))
            .collect();
        frame.render_widget(
            Paragraph::new(para_lines)
                .style(
                    Style::default()
                        .fg(self.theme.text_primary)
                        .bg(self.theme.body_background),
                )
                .scroll((self.details_scroll as u16, 0)),
            details_inner,
        );
        if text_lines.len() > details_inner.height as usize && details_inner.height > 0 {
            paint_scrollbar(
                frame,
                details_inner,
                self.details_scroll,
                text_lines.len(),
                self.theme.scrollbar,
                self.theme.scrollbar_hover,
                self.theme.body_background,
            );
        }
    }

    fn render_theme_editor(&self, frame: &mut ratatui::Frame, editor: &ldnddev_theme::ThemeEditor) {
        use ldnddev_theme::{ThemeEditorRow, theme_editor_rows};
        let area = centered_rect(80, 80, frame.area());
        frame.render_widget(Clear, area);
        let rows = theme_editor_rows(&editor.fields);
        let channel = ["R", "G", "B"][editor.channel.min(2)];
        let target = editor.save_target.label().to_uppercase();
        let mut lines = vec![
            Line::from(format!(
                "Save: {target} (Tab)   Channel: {channel} ([/])   Y save   R reset   Esc revert"
            )),
            Line::from(if editor.editing_hex {
                format!("Hex: {}█", editor.hex_draft)
            } else {
                format!("Hex: {}   Enter / +/- ", editor.hex_draft)
            }),
            Line::from(""),
        ];
        let view_h = area.height.saturating_sub(6) as usize;
        let start = rows
            .iter()
            .position(|row| match row {
                ThemeEditorRow::Color(idx) => *idx >= editor.scroll,
                ThemeEditorRow::Header(_) => false,
            })
            .unwrap_or(0);
        let start = if start > 0 && matches!(rows[start - 1], ThemeEditorRow::Header(_)) {
            start - 1
        } else {
            start
        };
        for row in rows.iter().skip(start).take(view_h.max(1)) {
            match row {
                ThemeEditorRow::Header(name) => lines.push(Line::from(*name)),
                ThemeEditorRow::Color(idx) => {
                    let field = editor.fields[*idx];
                    let hex = editor
                        .palette
                        .get(field.key)
                        .map(|c| c.to_hex())
                        .unwrap_or_else(|| "#000000".into());
                    let cursor = if *idx == editor.selected { ">" } else { " " };
                    lines.push(Line::from(format!("{cursor} {:<22} {hex}", field.key)));
                }
            }
        }
        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title("F2 Theme editor")
                    .borders(Borders::ALL)
                    .border_style(self.theme.active_border)
                    .style(
                        Style::default()
                            .bg(self.theme.modal_background)
                            .fg(self.theme.modal_text),
                    ),
            ),
            area,
        );
    }

    fn render_scroll_modal(&mut self, frame: &mut ratatui::Frame, title: &str, is_help: bool) {
        let area = centered_rect(80, 80, frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default()
            .title(title.to_string())
            .borders(Borders::ALL)
            .style(
                Style::default()
                    .fg(self.theme.modal_text)
                    .bg(self.theme.modal_background),
            )
            .border_style(Style::default().fg(self.theme.border_active))
            .title_style(
                Style::default()
                    .fg(self.theme.modal_header)
                    .add_modifier(Modifier::BOLD),
            );
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let scrollbar_width: u16 = 1;
        let body_w = inner.width.saturating_sub(scrollbar_width + 1);
        let body_area = Rect {
            x: inner.x,
            y: inner.y,
            width: body_w,
            height: inner.height,
        };

        let text = if is_help {
            build_help_text(&self.theme, body_w as usize)
        } else {
            build_theme_text(
                &self.theme,
                &self.theme_source,
                &self.theme_status,
                body_w as usize,
            )
        };
        let wrapped_total = count_wrapped_lines(&text, body_w as usize);
        let visible = inner.height as usize;
        let max_scroll = wrapped_total.saturating_sub(visible) as u16;
        if is_help {
            self.help_scroll_max = max_scroll;
            if self.help_scroll > max_scroll {
                self.help_scroll = max_scroll;
            }
        } else {
            self.theme_scroll_max = max_scroll;
            if self.theme_scroll > max_scroll {
                self.theme_scroll = max_scroll;
            }
        }
        let scroll = if is_help {
            self.help_scroll
        } else {
            self.theme_scroll
        };

        let body = Paragraph::new(text)
            .style(
                Style::default()
                    .fg(self.theme.modal_text)
                    .bg(self.theme.modal_background),
            )
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0));
        frame.render_widget(body, body_area);

        if (wrapped_total as u16) > inner.height {
            let track_x = inner.x + inner.width.saturating_sub(1);
            for row in 0..inner.height {
                let cell = Paragraph::new("│").style(
                    Style::default()
                        .fg(self.theme.scrollbar)
                        .bg(self.theme.modal_background),
                );
                frame.render_widget(
                    cell,
                    Rect {
                        x: track_x,
                        y: inner.y + row,
                        width: 1,
                        height: 1,
                    },
                );
            }
            let total_h = inner.height as usize;
            let thumb_h = ((total_h * total_h) / wrapped_total.max(1)).max(1);
            let scroll_range = wrapped_total.saturating_sub(total_h).max(1);
            let thumb_top = ((scroll as usize) * total_h.saturating_sub(thumb_h)) / scroll_range;
            for i in 0..thumb_h {
                let cell = Paragraph::new("█").style(
                    Style::default()
                        .fg(self.theme.scrollbar_hover)
                        .bg(self.theme.modal_background),
                );
                frame.render_widget(
                    cell,
                    Rect {
                        x: track_x,
                        y: inner.y + (thumb_top + i) as u16,
                        width: 1,
                        height: 1,
                    },
                );
            }
        }
    }
}

fn hit_at<'a>(hits: &'a [DetailHit], line_idx: usize, x: usize) -> Option<&'a DetailHit> {
    hits.iter()
        .rev()
        .find(|h| h.line == line_idx && x >= h.x0 && x < h.x1)
}

fn blueprint_cell_style(
    hit: Option<&DetailHit>,
    selected: Option<&TreeId>,
    theme: &super::theme::AppTheme,
) -> Style {
    let selected_here = match (selected, hit) {
        (Some(sel), Some(h)) => sel == &h.id,
        _ => false,
    };
    let bg = hit.and_then(|h| h.bg).unwrap_or(theme.body_background);
    let content_fg = hit.and_then(|h| h.fg).unwrap_or(theme.text_primary);
    let fg = if selected_here {
        theme.text_active_focus
    } else {
        readable_fg(content_fg, bg)
    };
    Style::default().fg(fg).bg(bg)
}

fn style_blueprint_line<'a>(
    line: &str,
    line_idx: usize,
    hits: &[DetailHit],
    selected: Option<&TreeId>,
    theme: &super::theme::AppTheme,
) -> Line<'a> {
    let chars: Vec<char> = line.chars().collect();
    if chars.is_empty() {
        return Line::from("");
    }
    let mut spans = Vec::new();
    let mut x = 0;
    while x < chars.len() {
        let style = blueprint_cell_style(hit_at(hits, line_idx, x), selected, theme);
        let mut x1 = x + 1;
        while x1 < chars.len() {
            let next = blueprint_cell_style(hit_at(hits, line_idx, x1), selected, theme);
            if next != style {
                break;
            }
            x1 += 1;
        }
        let text: String = chars[x..x1].iter().collect();
        spans.push(Span::styled(text, style));
        x = x1;
    }
    Line::from(spans)
}

fn paint_scrollbar(
    frame: &mut ratatui::Frame,
    inner: Rect,
    scroll: usize,
    total: usize,
    scrollbar: ratatui::style::Color,
    hover: ratatui::style::Color,
    bg: ratatui::style::Color,
) {
    if inner.height == 0 || inner.width == 0 {
        return;
    }
    let track_x = inner.x + inner.width.saturating_sub(1);
    for row in 0..inner.height {
        frame.render_widget(
            Paragraph::new("│").style(Style::default().fg(scrollbar).bg(bg)),
            Rect {
                x: track_x,
                y: inner.y + row,
                width: 1,
                height: 1,
            },
        );
    }
    let total_h = inner.height as usize;
    let thumb_h = ((total_h * total_h) / total.max(1)).max(1);
    let scroll_range = total.saturating_sub(total_h).max(1);
    let thumb_top = (scroll * total_h.saturating_sub(thumb_h)) / scroll_range;
    for i in 0..thumb_h {
        frame.render_widget(
            Paragraph::new("█").style(Style::default().fg(hover).bg(bg)),
            Rect {
                x: track_x,
                y: inner.y + (thumb_top + i) as u16,
                width: 1,
                height: 1,
            },
        );
    }
}

pub(super) fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
