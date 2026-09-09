use super::super::*;

impl App {
    pub(in crate::tui) fn render_modal(&self, frame: &mut ratatui::Frame) {
        let Some(modal) = &self.modal else {
            return;
        };
        match modal {
            Modal::LoadError { message } => {
                self.render_message_modal(frame, " Error ", message, "Enter / Esc: dismiss");
            }
            Modal::SavePrompt { path } => {
                self.render_message_modal(
                    frame,
                    " Save template ",
                    &format!("Save file path:\n{path}"),
                    "Enter or Ctrl+S: save  |  Esc: cancel",
                );
            }
            Modal::ConfirmPrompt { message, .. } => {
                self.render_message_modal(
                    frame,
                    " Confirm ",
                    &format!("{message}\n\ny = confirm, n / Esc = cancel"),
                    "",
                );
            }
            Modal::ValidationErrors {
                errors,
                scroll_offset,
            } => {
                self.render_validation_errors_modal(frame, errors, *scroll_offset);
            }
            Modal::MjmlMissing { searched } => {
                let msg = crate::mjml::not_found_message(searched);
                self.render_message_modal(frame, " MJML not found ", &msg, "Enter / Esc: dismiss");
            }
            Modal::MjmlCompileError { stderr, scroll } => {
                self.render_compile_error_modal(frame, stderr, *scroll);
            }
            Modal::FormEdit {
                state,
                cursor_pos,
                scroll_offset,
                ..
            } => {
                self.render_form_edit_modal(frame, state, *cursor_pos, *scroll_offset);
            }
            Modal::ComponentPicker { query, selected } => {
                self.render_component_picker(frame, query, *selected);
            }
            Modal::ImagePicker { state } => {
                self.render_image_picker(frame, state);
            }
        }
    }

    fn render_message_modal(
        &self,
        frame: &mut ratatui::Frame,
        title: &str,
        body: &str,
        footer: &str,
    ) {
        let area = centered_rect(70, 35, frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default()
            .title(title.to_string())
            .borders(Borders::ALL)
            .style(Style::default().bg(self.theme.modal_background))
            .border_style(Style::default().fg(self.theme.border_active))
            .title_style(
                Style::default()
                    .fg(self.theme.modal_header)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_widget(block.clone(), area);
        let inner = block.inner(area);
        let content = if footer.is_empty() {
            body.to_string()
        } else {
            format!("{body}\n\n{footer}")
        };
        frame.render_widget(
            Paragraph::new(content)
                .style(
                    Style::default()
                        .fg(self.theme.modal_text)
                        .bg(self.theme.modal_background),
                )
                .wrap(Wrap { trim: false }),
            inner,
        );
    }

    fn render_validation_errors_modal(
        &self,
        frame: &mut ratatui::Frame,
        errors: &[String],
        scroll_offset: usize,
    ) {
        let area = centered_rect(70, 60, frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default()
            .title(format!(" Validation — {} error(s) ", errors.len()))
            .borders(Borders::ALL)
            .style(Style::default().bg(self.theme.modal_background))
            .border_style(Style::default().fg(self.theme.border_active))
            .title_style(
                Style::default()
                    .fg(self.theme.modal_header)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_widget(block.clone(), area);
        let inner = block.inner(area);
        if inner.width < 4 || inner.height < 3 {
            return;
        }
        let padding_x: u16 = 2;
        let content_x = inner.x + padding_x;
        let content_w = inner.width.saturating_sub(padding_x * 2);
        let footer_height: u16 = 1;
        let list_height = inner.height.saturating_sub(footer_height);
        let lines: Vec<String> = errors.iter().map(|e| format!("- {e}")).collect();
        let visible: Vec<String> = lines
            .iter()
            .skip(scroll_offset)
            .take(list_height as usize)
            .cloned()
            .collect();
        frame.render_widget(
            Paragraph::new(visible.join("\n")).style(
                Style::default()
                    .fg(self.theme.modal_text)
                    .bg(self.theme.modal_background),
            ),
            Rect {
                x: content_x,
                y: inner.y,
                width: content_w,
                height: list_height,
            },
        );
        frame.render_widget(
            Paragraph::new("Enter / Esc: close  |  j/k: scroll").style(
                Style::default()
                    .fg(self.theme.modal_labels)
                    .bg(self.theme.modal_background),
            ),
            Rect {
                x: content_x,
                y: inner.y + list_height,
                width: content_w,
                height: 1,
            },
        );
    }

    fn render_compile_error_modal(&self, frame: &mut ratatui::Frame, stderr: &str, scroll: u16) {
        let area = centered_rect(80, 60, frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default()
            .title(" MJML compile error ")
            .borders(Borders::ALL)
            .style(Style::default().bg(self.theme.modal_background))
            .border_style(Style::default().fg(self.theme.border_active))
            .title_style(
                Style::default()
                    .fg(self.theme.modal_header)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_widget(block.clone(), area);
        let inner = block.inner(area);
        frame.render_widget(
            Paragraph::new(stderr.to_string())
                .style(
                    Style::default()
                        .fg(self.theme.modal_text)
                        .bg(self.theme.modal_background),
                )
                .wrap(Wrap { trim: false })
                .scroll((scroll, 0)),
            inner,
        );
    }

    pub(in crate::tui) fn render_form_edit_modal(
        &self,
        frame: &mut ratatui::Frame,
        state: &crate::tui::editform::EditFormState,
        cursor_pos: usize,
        scroll_offset: u16,
    ) {
        use crate::tui::editform;
        use crate::tui::form_textarea::*;

        self.form_expand_hits.borrow_mut().clear();
        if self.form_textarea_expanded {
            self.render_textarea_expand_modal(frame, state, cursor_pos);
            return;
        }

        let area = centered_rect(70, 80, frame.area());
        frame.render_widget(Clear, area);
        let outer = Block::default()
            .title(format!(" Edit — {} ", state.form.title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_active))
            .title_style(
                Style::default()
                    .fg(self.theme.modal_header)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(self.theme.modal_background));
        let inner = outer.inner(area);
        frame.render_widget(outer, area);
        if inner.height < 3 || inner.width < 6 {
            return;
        }

        let help_rect = Rect::new(inner.x, inner.y, inner.width, 1);
        let focused_is_textarea = matches!(
            state.form.fields.get(state.focused_field).map(|f| &f.kind),
            Some(editform::FieldKind::Textarea { .. })
        );
        let help_text = if focused_is_textarea {
            "Tab/Up/Down: navigate  |  Ctrl+E: expand  |  Ctrl+S: save  |  Esc: cancel"
        } else {
            "Tab/Up/Down: navigate  |  ←/→: cycle enum  |  Ctrl+S: save  |  Esc: cancel"
        };
        frame.render_widget(
            Paragraph::new(help_text).style(
                Style::default()
                    .fg(self.theme.modal_labels)
                    .bg(self.theme.modal_background)
                    .add_modifier(Modifier::BOLD),
            ),
            help_rect,
        );
        if inner.height < 4 {
            return;
        }
        let content_top = inner.y.saturating_add(2);
        let content_height = inner.height.saturating_sub(2);
        let scrollbar_col = inner.x.saturating_add(inner.width.saturating_sub(1));
        let content_rect = Rect::new(
            inner.x,
            content_top,
            inner.width.saturating_sub(1),
            content_height,
        );

        #[derive(Clone, Copy)]
        struct Slot {
            idx: usize,
            label_y: u16,
            box_y: u16,
            box_height: u16,
        }
        let mut slots: Vec<Slot> = Vec::new();
        let mut virt_y: u16 = 0;
        for (idx, field) in state.form.fields.iter().enumerate() {
            if !state.field_visible(field) {
                continue;
            }
            let content_rows: u16 = match &field.kind {
                editform::FieldKind::Textarea { rows, .. } => {
                    let max_rows = textarea_max_rows_for_window(content_height);
                    textarea_display_rows(
                        state.get(field.id),
                        (*rows).max(1),
                        Some(content_rect.width.saturating_sub(2)),
                        max_rows,
                    )
                }
                editform::FieldKind::SubForm { .. } => {
                    let items_len = state.sub_state.get(field.id).map(|v| v.len()).unwrap_or(0);
                    (1 + items_len.max(1)) as u16
                }
                _ => 1,
            };
            let box_height = content_rows.saturating_add(2);
            slots.push(Slot {
                idx,
                label_y: virt_y,
                box_y: virt_y.saturating_add(1),
                box_height,
            });
            virt_y = virt_y.saturating_add(1 + box_height + 1);
        }
        let total_height = virt_y;
        let max_scroll = total_height.saturating_sub(content_height);
        let scroll = scroll_offset.min(max_scroll);

        self.form_field_areas.borrow_mut().clear();

        for slot in &slots {
            let field = &state.form.fields[slot.idx];
            let focused = slot.idx == state.focused_field;
            let label_screen = slot.label_y as i32 - scroll as i32;
            let box_top_screen = slot.box_y as i32 - scroll as i32;
            let box_bottom_screen = box_top_screen + slot.box_height as i32;
            if box_bottom_screen <= 0 || label_screen >= content_height as i32 {
                continue;
            }
            if label_screen >= 0 && label_screen < content_height as i32 {
                let label_rect = Rect::new(
                    content_rect.x,
                    content_rect.y + label_screen as u16,
                    content_rect.width,
                    1,
                );
                let label_color = if focused {
                    self.theme.text_active_focus
                } else {
                    self.theme.modal_labels
                };
                let label_mod = if focused {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                };
                let mut label_spans = vec![Span::styled(
                    format!("{}:", field.label),
                    Style::default()
                        .fg(label_color)
                        .bg(self.theme.modal_background)
                        .add_modifier(label_mod),
                )];
                if let Some(hint) = field.hint {
                    label_spans.push(Span::styled(
                        format!("  {hint}"),
                        Style::default()
                            .fg(self.theme.text_secondary)
                            .bg(self.theme.modal_background),
                    ));
                }
                let is_textarea = matches!(field.kind, editform::FieldKind::Textarea { .. });
                let expand = "[Expand]";
                let expand_len = expand.len() as u16;
                if is_textarea && content_rect.width > expand_len + 2 {
                    let expand_x = content_rect
                        .x
                        .saturating_add(content_rect.width.saturating_sub(expand_len));
                    let label_width = content_rect.width.saturating_sub(expand_len + 1);
                    frame.render_widget(
                        Paragraph::new(Line::from(label_spans)),
                        Rect {
                            width: label_width,
                            ..label_rect
                        },
                    );
                    frame.render_widget(
                        Paragraph::new(expand).style(
                            Style::default()
                                .fg(self.theme.modal_labels)
                                .bg(self.theme.modal_background)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Rect {
                            x: expand_x,
                            y: label_rect.y,
                            width: expand_len,
                            height: 1,
                        },
                    );
                    self.form_expand_hits.borrow_mut().push((
                        slot.idx,
                        Rect {
                            x: expand_x,
                            y: label_rect.y,
                            width: expand_len,
                            height: 1,
                        },
                    ));
                } else {
                    frame.render_widget(Paragraph::new(Line::from(label_spans)), label_rect);
                }
            }
            if box_top_screen >= 0 && box_top_screen < content_height as i32 {
                let border_color = if focused {
                    self.theme.input_border_focus
                } else {
                    self.theme.input_border_default
                };
                let visible_box_height = slot
                    .box_height
                    .min(content_height.saturating_sub(box_top_screen as u16));
                if visible_box_height < 3 {
                    continue;
                }
                let box_rect = Rect::new(
                    content_rect.x,
                    content_rect.y + box_top_screen as u16,
                    content_rect.width,
                    visible_box_height,
                );
                let field_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(
                        Style::default()
                            .fg(border_color)
                            .bg(self.theme.modal_background),
                    )
                    .style(Style::default().bg(self.theme.modal_background));
                let inner_rect = field_block.inner(box_rect);
                frame.render_widget(field_block, box_rect);
                self.form_field_areas
                    .borrow_mut()
                    .push((box_rect, slot.idx));
                self.render_form_field_value(frame, field, state, cursor_pos, focused, inner_rect);
            }
        }

        if total_height > content_height {
            for row in 0..content_height {
                frame.render_widget(
                    Paragraph::new("│").style(
                        Style::default()
                            .fg(self.theme.scrollbar)
                            .bg(self.theme.modal_background),
                    ),
                    Rect {
                        x: scrollbar_col,
                        y: content_top + row,
                        width: 1,
                        height: 1,
                    },
                );
            }
            let thumb_height = ((content_height as u32 * content_height as u32
                / total_height.max(1) as u32) as u16)
                .max(1);
            let travel = content_height.saturating_sub(thumb_height);
            let thumb_y = if max_scroll == 0 {
                0
            } else {
                ((scroll as u32 * travel as u32) / max_scroll.max(1) as u32) as u16
            };
            for i in 0..thumb_height {
                frame.render_widget(
                    Paragraph::new("█").style(
                        Style::default()
                            .fg(self.theme.scrollbar_hover)
                            .bg(self.theme.modal_background),
                    ),
                    Rect {
                        x: scrollbar_col,
                        y: content_top + thumb_y + i,
                        width: 1,
                        height: 1,
                    },
                );
            }
        }
    }

    fn render_textarea_expand_modal(
        &self,
        frame: &mut ratatui::Frame,
        state: &crate::tui::editform::EditFormState,
        cursor_pos: usize,
    ) {
        let Some(field) = state.form.fields.get(state.focused_field) else {
            return;
        };
        let area = centered_rect(94, 90, frame.area());
        frame.render_widget(Clear, area);
        let outer = Block::default()
            .title(format!(" {} — expanded ", field.label))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_active))
            .title_style(
                Style::default()
                    .fg(self.theme.modal_header)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(self.theme.modal_background));
        let inner = outer.inner(area);
        frame.render_widget(outer, area);
        if inner.height < 4 || inner.width < 8 {
            return;
        }

        let help_rect = Rect::new(inner.x, inner.y, inner.width, 1);
        frame.render_widget(
            Paragraph::new("Esc: back to form  |  Ctrl+S: save  |  Enter: newline").style(
                Style::default()
                    .fg(self.theme.modal_labels)
                    .bg(self.theme.modal_background)
                    .add_modifier(Modifier::BOLD),
            ),
            help_rect,
        );

        let box_rect = Rect::new(
            inner.x,
            inner.y.saturating_add(2),
            inner.width,
            inner.height.saturating_sub(2),
        );
        if box_rect.height < 3 {
            return;
        }
        let field_block = Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(self.theme.input_border_focus)
                    .bg(self.theme.modal_background),
            )
            .style(Style::default().bg(self.theme.modal_background));
        let inner_rect = field_block.inner(box_rect);
        frame.render_widget(field_block, box_rect);
        self.form_field_areas.borrow_mut().clear();
        self.form_field_areas
            .borrow_mut()
            .push((box_rect, state.focused_field));
        self.render_form_field_value(frame, field, state, cursor_pos, true, inner_rect);
    }

    fn render_form_field_value(
        &self,
        frame: &mut ratatui::Frame,
        field: &crate::tui::editform::FormField,
        state: &crate::tui::editform::EditFormState,
        cursor_pos: usize,
        focused: bool,
        rect: Rect,
    ) {
        use crate::tui::editform;
        use crate::tui::form_textarea::*;

        let text_color = if focused {
            self.theme.input_text_focus
        } else {
            self.theme.input_text_default
        };
        let value_style = Style::default()
            .fg(text_color)
            .bg(self.theme.modal_background);

        match &field.kind {
            editform::FieldKind::Text { .. } | editform::FieldKind::Url { .. } => {
                let value = state.get(field.id);
                if value.is_empty() {
                    if let Some(placeholder) = field.placeholder {
                        frame.render_widget(
                            Paragraph::new(placeholder.to_string()).style(
                                Style::default()
                                    .fg(self.theme.text_disabled)
                                    .bg(self.theme.modal_background),
                            ),
                            rect,
                        );
                    }
                } else {
                    frame.render_widget(Paragraph::new(value.to_string()).style(value_style), rect);
                }
                if focused && rect.width > 0 && rect.height > 0 {
                    let col = cursor_pos.min(value.chars().count()) as u16;
                    if col < rect.width {
                        let ch = value.chars().nth(cursor_pos).unwrap_or(' ');
                        frame.render_widget(
                            Paragraph::new(ch.to_string()).style(
                                Style::default()
                                    .fg(self.theme.text_inverse)
                                    .bg(self.theme.cursor),
                            ),
                            Rect {
                                x: rect.x + col,
                                y: rect.y,
                                width: 1,
                                height: 1,
                            },
                        );
                    }
                }
            }
            editform::FieldKind::Textarea { .. } => {
                let value = state.get(field.id);
                let visible_rows = rect.height as usize;
                let (display, first_visible_row, total_rows) =
                    render_textarea_display_window(value, cursor_pos, focused, visible_rows);
                let text_rect = if total_rows > visible_rows {
                    Rect {
                        width: rect.width.saturating_sub(1),
                        ..rect
                    }
                } else {
                    rect
                };
                frame.render_widget(Paragraph::new(display).style(value_style), text_rect);
                if focused && text_rect.width > 0 && text_rect.height > 0 {
                    let row =
                        textarea_cursor_row(value, cursor_pos).saturating_sub(first_visible_row);
                    let col = textarea_cursor_col(value, cursor_pos);
                    if (row as u16) < text_rect.height && (col as u16) < text_rect.width {
                        let ch = value
                            .chars()
                            .nth(cursor_pos)
                            .filter(|c| *c != '\n')
                            .unwrap_or(' ');
                        frame.render_widget(
                            Paragraph::new(ch.to_string()).style(
                                Style::default()
                                    .fg(self.theme.text_inverse)
                                    .bg(self.theme.cursor),
                            ),
                            Rect {
                                x: text_rect.x + col as u16,
                                y: text_rect.y + row as u16,
                                width: 1,
                                height: 1,
                            },
                        );
                    }
                }
                if total_rows > visible_rows {
                    render_textarea_scrollbar(
                        frame,
                        Rect {
                            x: rect.x + rect.width.saturating_sub(1),
                            y: rect.y,
                            width: 1,
                            height: rect.height,
                        },
                        first_visible_row,
                        visible_rows,
                        total_rows,
                        self.theme.scrollbar,
                        self.theme.modal_background,
                    );
                }
            }
            editform::FieldKind::Enum { options, .. } => {
                let value = state.get(field.id);
                let display = format!("< {value} >");
                let mut style = value_style;
                if !options.iter().any(|o| *o == value) {
                    style = Style::default()
                        .fg(self.theme.error)
                        .bg(self.theme.modal_background);
                }
                frame.render_widget(Paragraph::new(display).style(style), rect);
            }
            editform::FieldKind::SubForm {
                summary_field_id, ..
            } => {
                let items = state.sub_state.get(field.id).cloned().unwrap_or_default();
                let selected = state.selected_sub_item.get(field.id).copied().unwrap_or(0);
                let mut lines: Vec<String> = Vec::new();
                lines.push(format!(
                    "{} item(s) — A add · X remove · Enter edit",
                    items.len()
                ));
                if items.is_empty() {
                    lines.push("  (no items; press A to add)".to_string());
                } else {
                    for (i, item) in items.iter().enumerate() {
                        let summary = item
                            .values
                            .get(*summary_field_id)
                            .cloned()
                            .unwrap_or_default();
                        let summary = if summary.trim().is_empty() {
                            "(untitled)".to_string()
                        } else {
                            summary
                        };
                        let marker = if focused && i == selected { ">" } else { " " };
                        lines.push(format!("  {marker} {}. {summary}", i + 1));
                    }
                }
                frame.render_widget(Paragraph::new(lines.join("\n")).style(value_style), rect);
            }
        }
    }

    fn render_component_picker(&self, frame: &mut ratatui::Frame, query: &str, selected: usize) {
        let area = centered_rect(70, 70, frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default()
            .title(" Insert component ")
            .borders(Borders::ALL)
            .style(Style::default().bg(self.theme.modal_background))
            .border_style(Style::default().fg(self.theme.border_active))
            .title_style(
                Style::default()
                    .fg(self.theme.modal_header)
                    .add_modifier(Modifier::BOLD),
            );
        let inner = block.inner(area);
        frame.render_widget(block, area);
        if inner.height < 4 {
            return;
        }
        frame.render_widget(
            Paragraph::new(format!("Search: {query}_")).style(
                Style::default()
                    .fg(self.theme.text_active_focus)
                    .bg(self.theme.modal_background),
            ),
            Rect::new(inner.x, inner.y, inner.width, 1),
        );
        let class = crate::tui::component_kind::classify(
            self.template.as_ref(),
            &self
                .selected_tree_id()
                .unwrap_or(crate::tui::tree::TreeId::Body),
        );
        let rows = crate::tui::component_kind::picker_rows(class, query);
        let body_y = inner.y + 2;
        let body_h = inner.height.saturating_sub(3);
        if rows.is_empty() {
            frame.render_widget(
                Paragraph::new("(nothing can be inserted here)").style(
                    Style::default()
                        .fg(self.theme.text_secondary)
                        .bg(self.theme.modal_background),
                ),
                Rect::new(inner.x, body_y, inner.width, 1),
            );
        } else {
            let visible = body_h as usize;
            let start = if selected >= visible {
                selected + 1 - visible
            } else {
                0
            };
            for (i, row) in rows.iter().skip(start).take(visible).enumerate() {
                let y = body_y + i as u16;
                let is_sel = start + i == selected;
                let (text, style) = match row {
                    crate::tui::component_kind::PickerRow::Header(h) => (
                        (*h).to_string(),
                        Style::default()
                            .fg(self.theme.modal_labels)
                            .bg(self.theme.modal_background)
                            .add_modifier(Modifier::BOLD),
                    ),
                    crate::tui::component_kind::PickerRow::Kind(k) => {
                        let label = format!("  {}", k.label());
                        if is_sel {
                            (
                                label,
                                Style::default()
                                    .fg(self.theme.text_active_focus)
                                    .bg(self.theme.selected_background),
                            )
                        } else {
                            (
                                label,
                                Style::default()
                                    .fg(self.theme.modal_text)
                                    .bg(self.theme.modal_background),
                            )
                        }
                    }
                };
                frame.render_widget(
                    Paragraph::new(text).style(style),
                    Rect::new(inner.x, y, inner.width, 1),
                );
            }
        }
        frame.render_widget(
            Paragraph::new("Type to filter  |  Up/Down: select  |  Enter: insert  |  Esc: cancel")
                .style(
                    Style::default()
                        .fg(self.theme.modal_labels)
                        .bg(self.theme.modal_background),
                ),
            Rect::new(
                inner.x,
                inner.y + inner.height.saturating_sub(1),
                inner.width,
                1,
            ),
        );
    }

    fn render_image_picker(&self, frame: &mut ratatui::Frame, state: &ImagePickerState) {
        use crate::tui::util::{filter_entries, list_dir_entries};
        let area = centered_rect(70, 70, frame.area());
        frame.render_widget(Clear, area);
        let outer = Block::default()
            .title(" Pick image ")
            .borders(Borders::ALL)
            .style(Style::default().bg(self.theme.modal_background))
            .border_style(Style::default().fg(self.theme.border_active))
            .title_style(
                Style::default()
                    .fg(self.theme.modal_header)
                    .add_modifier(Modifier::BOLD),
            );
        let inner = outer.inner(area);
        frame.render_widget(outer, area);
        if inner.height < 5 || inner.width < 10 {
            return;
        }
        let pad: u16 = 2;
        let content_x = inner.x + pad;
        let content_w = inner.width.saturating_sub(pad * 2);
        let rel = state.cwd.strip_prefix(&state.root).unwrap_or(&state.cwd);
        let rel_str = rel.to_string_lossy();
        let cwd_label = if rel_str.is_empty() {
            "Folder: ./".to_string()
        } else {
            format!("Folder: ./{rel_str}")
        };
        frame.render_widget(
            Paragraph::new(cwd_label).style(
                Style::default()
                    .fg(self.theme.text_secondary)
                    .bg(self.theme.modal_background),
            ),
            Rect::new(content_x, inner.y, content_w, 1),
        );
        frame.render_widget(
            Paragraph::new(format!("Filter: {}_", state.filter)).style(
                Style::default()
                    .fg(self.theme.text_active_focus)
                    .bg(self.theme.modal_background),
            ),
            Rect::new(content_x, inner.y + 1, content_w, 1),
        );
        let entries = list_dir_entries(&state.cwd);
        let filtered = filter_entries(&entries, &state.filter);
        let body_y = inner.y + 3;
        let body_h = inner.height.saturating_sub(5);
        let visible = body_h as usize;
        let start = if filtered.is_empty() {
            0
        } else if state.selected >= visible {
            state.selected + 1 - visible
        } else {
            0
        };
        if filtered.is_empty() {
            frame.render_widget(
                Paragraph::new("(no matches)").style(
                    Style::default()
                        .fg(self.theme.text_secondary)
                        .bg(self.theme.modal_background),
                ),
                Rect::new(content_x, body_y, content_w, 1),
            );
        } else {
            for (i, entry) in filtered.iter().skip(start).take(visible).enumerate() {
                let row = body_y + i as u16;
                let is_selected = (start + i) == state.selected;
                let glyph = if entry.is_dir { "/" } else { " " };
                let line = format!("{glyph} {}", entry.name);
                let (fg, bg) = if is_selected {
                    (self.theme.text_active_focus, self.theme.selected_background)
                } else if entry.is_dir {
                    (self.theme.folders, self.theme.modal_background)
                } else {
                    (self.theme.files, self.theme.modal_background)
                };
                frame.render_widget(
                    Paragraph::new(line).style(Style::default().fg(fg).bg(bg)),
                    Rect::new(content_x, row, content_w, 1),
                );
            }
        }
        frame.render_widget(
            Paragraph::new("↑/↓ move  |  ← parent  |  →/Enter pick  |  type filter  |  Esc cancel")
                .style(
                    Style::default()
                        .fg(self.theme.modal_labels)
                        .bg(self.theme.modal_background),
                ),
            Rect::new(
                content_x,
                inner.y + inner.height.saturating_sub(1),
                content_w,
                1,
            ),
        );
    }
}
