//! Frame layout: header, body, footer, Settings sheet, modals, toasts.
use super::details::{DetailHit, readable_fg};
use super::help::build_help_text;
use super::tree::TreeId;
use super::*;

impl App {
    pub(super) fn draw(&mut self, frame: &mut ratatui::Frame) {
        self.prune_toasts();
        self.settings_nav_area = Rect::default();

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

        let confirm_up = matches!(self.modal, Some(Modal::ConfirmPrompt { .. }));
        if !confirm_up {
            self.render_modal(frame);
        }
        if self.show_theme {
            self.render_settings(frame, root[1]);
        }
        if confirm_up {
            self.render_modal(frame);
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

        if self.template.is_none() {
            let muted = Style::default()
                .fg(self.theme.text_secondary)
                .bg(self.theme.body_background);
            let key = Style::default()
                .fg(self.theme.text_active_focus)
                .bg(self.theme.body_background);
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(Span::styled("No template open.", muted)),
                    Line::from(""),
                    Line::from(Span::styled("n  New", key)),
                    Line::from(Span::styled("o  Open", key)),
                ])
                .style(
                    Style::default()
                        .fg(self.theme.text_primary)
                        .bg(self.theme.body_background),
                ),
                tree_inner,
            );
        } else {
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
        let details_heading = if self.template.is_none() {
            "Details".to_string()
        } else {
            details_title(&selected_label(&rows, self.selected_row))
        };
        let details_block = Block::default()
            .title(details_heading)
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

    fn render_settings(&mut self, frame: &mut ratatui::Frame, body: Rect) {
        use ldnddev_theme::{PickerRow, ThemeEditorRow, theme_editor_rows};
        if body.width == 0 || body.height == 0 {
            return;
        }
        let width = (body.width / 2).max(1).min(body.width);
        let area = Rect {
            x: body.x + body.width.saturating_sub(width),
            y: body.y,
            width,
            height: body.height,
        };
        frame.render_widget(Clear, area);
        let block = Block::default()
            .title(" Settings ")
            .borders(Borders::ALL)
            .border_style(self.theme.active_border)
            .title_style(
                Style::default()
                    .fg(self.theme.modal_header)
                    .add_modifier(Modifier::BOLD),
            )
            .style(
                Style::default()
                    .bg(self.theme.modal_background)
                    .fg(self.theme.modal_text),
            );
        let inner = block.inner(area);
        frame.render_widget(block, area);
        if inner.width == 0 || inner.height == 0 {
            return;
        }
        if self.settings_nav as usize >= SETTINGS_NAV.len() {
            self.settings_nav = SETTINGS_THEMES;
        }

        let nav_w = {
            let spare = inner.width.saturating_sub(8);
            if spare == 0 {
                1.min(inner.width)
            } else {
                14.min(spare).max(1)
            }
        };
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(nav_w), Constraint::Min(1)])
            .split(inner);
        let nav = cols[0];
        let page = cols[1];
        self.settings_nav_area = nav;
        paint_settings_nav(frame, self, nav);

        let section = SETTINGS_NAV
            .get(self.settings_nav as usize)
            .copied()
            .unwrap_or("Themes");
        let title_h = u16::from(page.height > 0);
        let title_area = Rect {
            x: page.x,
            y: page.y,
            width: page.width,
            height: title_h,
        };
        let content = Rect {
            x: page.x,
            y: page.y.saturating_add(title_h),
            width: page.width,
            height: page.height.saturating_sub(title_h),
        };
        if title_h > 0 {
            let shown: String = section.chars().take(page.width as usize).collect();
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    shown,
                    Style::default()
                        .fg(self.theme.modal_header)
                        .add_modifier(Modifier::BOLD),
                )))
                .style(
                    Style::default()
                        .bg(self.theme.modal_background)
                        .fg(self.theme.modal_header),
                ),
                title_area,
            );
        }

        if self.settings_nav == SETTINGS_HELP {
            let lines = build_help_text(&self.theme, content.width.max(1) as usize).lines;
            self.paint_settings_lines(frame, content, lines, None);
            return;
        }

        let editor = self.theme_editor.clone();
        let picker = self.theme_picker.clone();
        let box_w = (content.width as usize).max(2);
        let mut lines: Vec<Line<'static>> = Vec::new();
        let mut selected_line = None;
        let rule = Style::default().fg(self.theme.border_default);
        let header = Style::default()
            .fg(self.theme.modal_header)
            .add_modifier(Modifier::BOLD);
        let hint = if let Some(editor) = &editor {
            let target = editor.save_target.label();
            if picker.is_some() {
                format!("Tab section   j/k move   t target   y save {target}   R reset   Esc back")
            } else {
                format!(
                    "Tab section   j/k move   t target   y save {target}   R reset   Esc revert"
                )
            }
        } else {
            "Tab section   j/k move   Enter apply   e colors   y save   t target   Esc revert"
                .to_string()
        };
        lines.push(Line::from(Span::styled(
            hint,
            Style::default().fg(self.theme.text_secondary),
        )));

        if let Some(editor) = &editor {
            if editor.editing_hex {
                lines.push(Line::from(format!("Hex: {}█", editor.hex_draft)));
            } else {
                lines.push(Line::from(format!(
                    "Hex: {}   Enter edit",
                    editor.hex_draft
                )));
            }
            let selected_style = Style::default()
                .fg(self.theme.text_active_focus)
                .bg(self.theme.selected_background);
            let idle = Style::default().fg(self.theme.modal_text);
            let mut open = false;
            for row in theme_editor_rows(&editor.fields) {
                match row {
                    ThemeEditorRow::Header(name) => {
                        if open {
                            lines.push(settings_bottom(box_w, rule));
                        }
                        lines.push(Line::from(""));
                        lines.push(settings_top(name, box_w, rule, header));
                        open = true;
                    }
                    ThemeEditorRow::Color(idx) => {
                        if idx == editor.selected {
                            selected_line = Some(lines.len());
                        }
                        let field = editor.fields[idx];
                        let hex = editor
                            .palette
                            .get(field.key)
                            .map(|color| color.to_hex())
                            .unwrap_or_else(|| "#000000".into());
                        let cursor = if idx == editor.selected { ">" } else { " " };
                        let style = if idx == editor.selected {
                            selected_style
                        } else {
                            idle
                        };
                        lines.push(settings_row(
                            &format!("{cursor} {:<22} {hex}", field.key),
                            box_w,
                            rule,
                            style,
                        ));
                    }
                }
            }
            if open {
                lines.push(settings_bottom(box_w, rule));
            }
        } else if let Some(picker) = &picker {
            lines.push(Line::from(""));
            lines.push(settings_top("Themes", box_w, rule, header));
            let selected_style = Style::default()
                .fg(self.theme.text_active_focus)
                .bg(self.theme.selected_background);
            let idle = Style::default().fg(self.theme.modal_text);
            for (idx, row) in picker.rows.iter().enumerate() {
                if idx == picker.selected {
                    selected_line = Some(lines.len());
                }
                let (label, detail) = match row {
                    PickerRow::Follow { name } => ("Follow desktop".to_string(), name.clone()),
                    PickerRow::Theme(theme) => (theme.name.clone(), theme.mode.label().to_string()),
                    PickerRow::Custom => ("Custom".to_string(), "built-in".to_string()),
                };
                let cursor = if idx == picker.selected { ">" } else { " " };
                let style = if idx == picker.selected {
                    selected_style
                } else {
                    idle
                };
                lines.push(settings_row(
                    &format!("{cursor} {label:<22} {detail}"),
                    box_w,
                    rule,
                    style,
                ));
            }
            lines.push(settings_bottom(box_w, rule));
        }

        self.paint_settings_lines(frame, content, lines, selected_line);
    }

    fn paint_settings_lines(
        &mut self,
        frame: &mut ratatui::Frame,
        area: Rect,
        lines: Vec<Line<'static>>,
        selected_line: Option<usize>,
    ) {
        if area.width == 0 || area.height == 0 {
            self.help_scroll_max = 0;
            self.help_scroll = 0;
            return;
        }
        let view_h = area.height as usize;
        let max_scroll = lines.len().saturating_sub(view_h) as u16;
        self.help_scroll_max = max_scroll;
        if self.settings_follow {
            if let Some(sel) = selected_line {
                let top = self.help_scroll as usize;
                if sel < top {
                    self.help_scroll = sel as u16;
                } else if sel >= top + view_h {
                    self.help_scroll = (sel + 1 - view_h) as u16;
                }
            }
            self.settings_follow = false;
        }
        if self.help_scroll > max_scroll {
            self.help_scroll = max_scroll;
        }
        frame.render_widget(
            Paragraph::new(lines)
                .style(
                    Style::default()
                        .fg(self.theme.modal_text)
                        .bg(self.theme.modal_background),
                )
                .scroll((self.help_scroll, 0)),
            area,
        );
    }
}

fn paint_settings_nav(frame: &mut ratatui::Frame, app: &App, nav: Rect) {
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(app.theme.border_default))
        .style(
            Style::default()
                .bg(app.theme.body_background)
                .fg(app.theme.modal_text),
        );
    let inner = block.inner(nav);
    frame.render_widget(block, nav);
    if inner.width == 0 || inner.height == 0 {
        return;
    }
    let selected_style = Style::default()
        .fg(app.theme.text_active_focus)
        .bg(app.theme.selected_background);
    let idle = Style::default()
        .fg(app.theme.modal_text)
        .bg(app.theme.body_background);
    let width = inner.width as usize;
    let mut lines = Vec::new();
    for (index, label) in SETTINGS_NAV.iter().enumerate() {
        let on = app.settings_nav == index as u8;
        let marker = if on { ">" } else { " " };
        let text: String = format!("{marker} {label}").chars().take(width).collect();
        let style = if on { selected_style } else { idle };
        lines.push(Line::from(Span::styled(text, style)));
    }
    frame.render_widget(
        Paragraph::new(lines).style(
            Style::default()
                .bg(app.theme.body_background)
                .fg(app.theme.modal_text),
        ),
        inner,
    );
}

fn settings_top(title: &str, width: usize, edge: Style, title_style: Style) -> Line<'static> {
    let width = width.max(2);
    let inner = width - 2;
    let mut label = format!(" {title} ");
    if label.chars().count() > inner {
        label = label.chars().take(inner).collect();
    }
    let pad = inner - label.chars().count();
    Line::from(vec![
        Span::styled("┌", edge),
        Span::styled(label, title_style),
        Span::styled(format!("{}┐", "─".repeat(pad)), edge),
    ])
}

fn settings_bottom(width: usize, edge: Style) -> Line<'static> {
    let width = width.max(2);
    Line::from(Span::styled(format!("└{}┘", "─".repeat(width - 2)), edge))
}

fn settings_row(body: &str, width: usize, edge: Style, fill: Style) -> Line<'static> {
    let width = width.max(2);
    let inner = width - 2;
    let mut text: String = body.chars().take(inner).collect();
    let pad = inner - text.chars().count();
    text.push_str(&" ".repeat(pad));
    Line::from(vec![
        Span::styled("│", edge),
        Span::styled(text, fill),
        Span::styled("│", edge),
    ])
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
