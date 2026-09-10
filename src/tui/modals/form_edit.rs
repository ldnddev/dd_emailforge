//! FormEdit key handling, drill-down, and field edits.
use super::super::*;

#[derive(Debug, Clone)]
pub(in crate::tui) struct DrillFrame {
    pub parent_state: super::super::editform::EditFormState,
    pub parent_cursor_pos: usize,
    pub parent_scroll_offset: u16,
    pub subform_field_id: String,
    pub item_idx: usize,
}

impl App {
    pub(in crate::tui) fn handle_form_edit_event(
        &mut self,
        key: event::KeyEvent,
    ) -> Option<ModalResult> {
        use crate::tui::editform;
        use crate::tui::form_textarea::*;

        if matches!(key.code, KeyCode::Char('e')) && key.modifiers.contains(KeyModifiers::CONTROL) {
            let is_textarea = matches!(
                self.modal.as_ref(),
                Some(Modal::FormEdit { state, .. })
                    if matches!(
                        state.form.fields.get(state.focused_field).map(|f| &f.kind),
                        Some(editform::FieldKind::Textarea { .. })
                    )
            );
            if is_textarea {
                self.form_textarea_expanded = !self.form_textarea_expanded;
            }
            return Some(ModalResult::Continue);
        }

        if matches!(key.code, KeyCode::Char('p')) && key.modifiers.contains(KeyModifiers::CONTROL) {
            let Some(Modal::FormEdit { state, .. }) = self.modal.as_ref() else {
                return Some(ModalResult::Continue);
            };
            let field_id = match state.form.fields.get(state.focused_field) {
                Some(f)
                    if matches!(f.kind, editform::FieldKind::Url { .. })
                        && super::pickers::is_image_url_field(f.id) =>
                {
                    f.id.to_string()
                }
                _ => return Some(ModalResult::Continue),
            };
            self.open_image_picker(field_id);
            return Some(ModalResult::Continue);
        }

        if matches!(key.code, KeyCode::Char('s')) && key.modifiers.contains(KeyModifiers::CONTROL) {
            let taken = self.modal.take();
            if let Some(Modal::FormEdit {
                state,
                cursor,
                cursor_pos,
                mut drill_stack,
                scroll_offset: _,
            }) = taken
            {
                if let Some(frame) = drill_stack.pop() {
                    let mut parent = frame.parent_state;
                    let items = parent
                        .sub_state
                        .entry(frame.subform_field_id.clone())
                        .or_default();
                    if frame.item_idx < items.len() {
                        items[frame.item_idx] = state;
                    } else {
                        items.push(state);
                    }
                    self.form_textarea_expanded = false;
                    self.push_toast(ToastLevel::Success, "Item saved — editing parent.");
                    self.modal = Some(Modal::FormEdit {
                        state: parent,
                        cursor,
                        cursor_pos: frame.parent_cursor_pos,
                        drill_stack,
                        scroll_offset: frame.parent_scroll_offset,
                    });
                    return Some(ModalResult::Continue);
                }
                let Some(template) = self.template.as_mut() else {
                    self.form_textarea_expanded = false;
                    self.push_toast(ToastLevel::Warning, "No template open.");
                    return Some(ModalResult::CloseCancel);
                };
                match crate::tui::cursor::apply_form(template, &cursor, &state) {
                    Ok(()) => {
                        self.form_textarea_expanded = false;
                        let msg = format!("Saved {}.", state.form.title);
                        self.push_toast(ToastLevel::Success, msg);
                        return Some(ModalResult::CloseSuccess);
                    }
                    Err(e) => {
                        self.push_toast(ToastLevel::Warning, format!("Save failed: {e}"));
                        self.modal = Some(Modal::FormEdit {
                            state,
                            cursor,
                            cursor_pos,
                            drill_stack,
                            scroll_offset: 0,
                        });
                        return Some(ModalResult::Continue);
                    }
                }
            }
            return Some(ModalResult::CloseCancel);
        }

        if matches!(key.code, KeyCode::Esc) {
            if self.form_textarea_expanded {
                self.form_textarea_expanded = false;
                return Some(ModalResult::Continue);
            }
            let taken = self.modal.take();
            if let Some(Modal::FormEdit {
                cursor,
                mut drill_stack,
                ..
            }) = taken
            {
                if let Some(frame) = drill_stack.pop() {
                    self.push_toast(ToastLevel::Info, "Item edit cancelled.");
                    self.modal = Some(Modal::FormEdit {
                        state: frame.parent_state,
                        cursor,
                        cursor_pos: frame.parent_cursor_pos,
                        drill_stack,
                        scroll_offset: frame.parent_scroll_offset,
                    });
                    return Some(ModalResult::Continue);
                }
            }
            self.form_textarea_expanded = false;
            self.modal = None;
            return Some(ModalResult::CloseCancel);
        }

        let expanded = self.form_textarea_expanded;
        let Some(Modal::FormEdit {
            state,
            cursor_pos,
            scroll_offset,
            ..
        }) = self.modal.as_mut()
        else {
            return Some(ModalResult::CloseCancel);
        };

        let focused_idx = state.focused_field;
        let (field_id, is_enum, is_textarea, is_subform, is_checkboxes, accepts_text) =
            match state.form.fields.get(focused_idx) {
                Some(f) => (
                    f.id,
                    matches!(f.kind, editform::FieldKind::Enum { .. }),
                    matches!(f.kind, editform::FieldKind::Textarea { .. }),
                    matches!(f.kind, editform::FieldKind::SubForm { .. }),
                    matches!(f.kind, editform::FieldKind::Checkboxes { .. }),
                    matches!(
                        f.kind,
                        editform::FieldKind::Text { .. }
                            | editform::FieldKind::Url { .. }
                            | editform::FieldKind::Textarea { .. }
                    ),
                ),
                None => return Some(ModalResult::CloseCancel),
            };

        if is_subform {
            match key.code {
                KeyCode::Char('A') => {
                    if let Some(new_item) = state.new_sub_item(field_id) {
                        let items = state.sub_state.entry(field_id.to_string()).or_default();
                        let selected = state.selected_sub_item.get(field_id).copied().unwrap_or(0);
                        let insert_at = if items.is_empty() {
                            0
                        } else {
                            (selected + 1).min(items.len())
                        };
                        items.insert(insert_at, new_item);
                        state
                            .selected_sub_item
                            .insert(field_id.to_string(), insert_at);
                        self.push_toast(ToastLevel::Success, "Item added.");
                    }
                    return Some(ModalResult::Continue);
                }
                KeyCode::Char('X') => {
                    let min_items = match state.form.fields[focused_idx].kind {
                        editform::FieldKind::SubForm { min_items, .. } => min_items,
                        _ => 0,
                    };
                    let items = state.sub_state.entry(field_id.to_string()).or_default();
                    if items.len() > min_items {
                        let selected = state.selected_sub_item.get(field_id).copied().unwrap_or(0);
                        if selected < items.len() {
                            items.remove(selected);
                            let new_sel = selected.min(items.len().saturating_sub(1));
                            state
                                .selected_sub_item
                                .insert(field_id.to_string(), new_sel);
                            self.push_toast(ToastLevel::Info, "Item removed.");
                        }
                    } else {
                        self.push_toast(
                            ToastLevel::Warning,
                            format!("Must keep at least {min_items} item(s)."),
                        );
                    }
                    return Some(ModalResult::Continue);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let selected = state.selected_sub_item.get(field_id).copied().unwrap_or(0);
                    let items_len = state.sub_state.get(field_id).map(|v| v.len()).unwrap_or(0);
                    if items_len == 0 || selected == 0 {
                        state.focus_prev();
                        *scroll_offset = auto_scroll_for_focus(state, *scroll_offset);
                        *cursor_pos = state
                            .get(state.form.fields[state.focused_field].id)
                            .chars()
                            .count();
                    } else {
                        state
                            .selected_sub_item
                            .insert(field_id.to_string(), selected - 1);
                    }
                    return Some(ModalResult::Continue);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let selected = state.selected_sub_item.get(field_id).copied().unwrap_or(0);
                    let items_len = state.sub_state.get(field_id).map(|v| v.len()).unwrap_or(0);
                    if selected + 1 < items_len {
                        state
                            .selected_sub_item
                            .insert(field_id.to_string(), selected + 1);
                    } else {
                        state.focus_next();
                        *scroll_offset = auto_scroll_for_focus(state, *scroll_offset);
                        *cursor_pos = state
                            .get(state.form.fields[state.focused_field].id)
                            .chars()
                            .count();
                    }
                    return Some(ModalResult::Continue);
                }
                KeyCode::Enter => {
                    let taken = self.modal.take();
                    if let Some(Modal::FormEdit {
                        mut state,
                        cursor,
                        cursor_pos,
                        mut drill_stack,
                        scroll_offset,
                    }) = taken
                    {
                        let selected = state.selected_sub_item.get(field_id).copied().unwrap_or(0);
                        let items_len = state.sub_state.get(field_id).map(|v| v.len()).unwrap_or(0);
                        if selected < items_len {
                            let template = match &state.form.fields[focused_idx].kind {
                                editform::FieldKind::SubForm { template, .. } => *template,
                                _ => unreachable!("is_subform was true"),
                            };
                            let placeholder = editform::EditFormState::new(template);
                            let items = state
                                .sub_state
                                .get_mut(field_id)
                                .expect("sub_state present for SubForm field");
                            let item_state = std::mem::replace(&mut items[selected], placeholder);
                            let item_cursor_pos = item_state
                                .get(item_state.form.fields[item_state.focused_field].id)
                                .chars()
                                .count();
                            drill_stack.push(DrillFrame {
                                parent_state: state,
                                parent_cursor_pos: cursor_pos,
                                parent_scroll_offset: scroll_offset,
                                subform_field_id: field_id.to_string(),
                                item_idx: selected,
                            });
                            self.modal = Some(Modal::FormEdit {
                                state: item_state,
                                cursor,
                                cursor_pos: item_cursor_pos,
                                drill_stack,
                                scroll_offset: 0,
                            });
                            self.push_toast(
                                ToastLevel::Info,
                                "Editing item. Ctrl+S returns to parent.",
                            );
                        } else {
                            self.modal = Some(Modal::FormEdit {
                                state,
                                cursor,
                                cursor_pos,
                                drill_stack,
                                scroll_offset,
                            });
                        }
                    }
                    return Some(ModalResult::Continue);
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Tab if !expanded => {
                state.focus_next();
                *scroll_offset = auto_scroll_for_focus(state, *scroll_offset);
                *cursor_pos = state
                    .get(state.form.fields[state.focused_field].id)
                    .chars()
                    .count();
            }
            KeyCode::BackTab if !expanded => {
                state.focus_prev();
                *scroll_offset = auto_scroll_for_focus(state, *scroll_offset);
                *cursor_pos = state
                    .get(state.form.fields[state.focused_field].id)
                    .chars()
                    .count();
            }
            KeyCode::Left => {
                if is_enum {
                    state.cycle_enum(false);
                } else if is_checkboxes {
                    state.move_checkbox_cursor(-1);
                } else if *cursor_pos > 0 {
                    *cursor_pos -= 1;
                }
            }
            KeyCode::Right => {
                if is_enum {
                    state.cycle_enum(true);
                } else if is_checkboxes {
                    state.move_checkbox_cursor(1);
                } else {
                    let len = state.get(field_id).chars().count();
                    if *cursor_pos < len {
                        *cursor_pos += 1;
                    }
                }
            }
            KeyCode::Up => {
                if is_textarea {
                    let wrap = textarea_wrap_width(&self.form_textarea_hits, focused_idx);
                    *cursor_pos =
                        textarea_move_cursor_vertical(state.get(field_id), *cursor_pos, -1, wrap);
                } else {
                    state.focus_prev();
                    *scroll_offset = auto_scroll_for_focus(state, *scroll_offset);
                    *cursor_pos = state
                        .get(state.form.fields[state.focused_field].id)
                        .chars()
                        .count();
                }
            }
            KeyCode::Down => {
                if is_textarea {
                    let wrap = textarea_wrap_width(&self.form_textarea_hits, focused_idx);
                    *cursor_pos =
                        textarea_move_cursor_vertical(state.get(field_id), *cursor_pos, 1, wrap);
                } else {
                    state.focus_next();
                    *scroll_offset = auto_scroll_for_focus(state, *scroll_offset);
                    *cursor_pos = state
                        .get(state.form.fields[state.focused_field].id)
                        .chars()
                        .count();
                }
            }
            KeyCode::PageUp if is_textarea => {
                let wrap = textarea_wrap_width(&self.form_textarea_hits, focused_idx);
                *cursor_pos =
                    textarea_move_cursor_vertical(state.get(field_id), *cursor_pos, -10, wrap);
            }
            KeyCode::PageDown if is_textarea => {
                let wrap = textarea_wrap_width(&self.form_textarea_hits, focused_idx);
                *cursor_pos =
                    textarea_move_cursor_vertical(state.get(field_id), *cursor_pos, 10, wrap);
            }
            KeyCode::Home if is_textarea => {
                let wrap = textarea_wrap_width(&self.form_textarea_hits, focused_idx);
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    *cursor_pos = 0;
                } else {
                    *cursor_pos = textarea_home(state.get(field_id), *cursor_pos, wrap);
                }
            }
            KeyCode::End if is_textarea => {
                let wrap = textarea_wrap_width(&self.form_textarea_hits, focused_idx);
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    *cursor_pos = state.get(field_id).chars().count();
                } else {
                    *cursor_pos = textarea_end(state.get(field_id), *cursor_pos, wrap);
                }
            }
            KeyCode::Char(' ') if is_checkboxes => {
                state.toggle_focused_checkbox();
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                if accepts_text {
                    let current = state.get(field_id).to_string();
                    let (new, pos) = insert_char(&current, *cursor_pos, c);
                    state.set(field_id, new);
                    *cursor_pos = pos;
                }
            }
            KeyCode::Backspace => {
                if accepts_text {
                    let current = state.get(field_id).to_string();
                    let (new, pos) = delete_char_before(&current, *cursor_pos);
                    state.set(field_id, new);
                    *cursor_pos = pos;
                }
            }
            KeyCode::Enter => {
                if is_textarea {
                    let current = state.get(field_id).to_string();
                    let (new, pos) = insert_char(&current, *cursor_pos, '\n');
                    state.set(field_id, new);
                    *cursor_pos = pos;
                } else {
                    state.focus_next();
                    *scroll_offset = auto_scroll_for_focus(state, *scroll_offset);
                    *cursor_pos = state
                        .get(state.form.fields[state.focused_field].id)
                        .chars()
                        .count();
                }
            }
            _ => {}
        }

        Some(ModalResult::Continue)
    }

    pub(in crate::tui) fn handle_form_edit_mouse(
        &mut self,
        m: event::MouseEvent,
    ) -> Option<ModalResult> {
        match m.kind {
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                let delta: isize = if matches!(m.kind, MouseEventKind::ScrollUp) {
                    -3
                } else {
                    3
                };
                if self.form_textarea_expanded {
                    if let Some(Modal::FormEdit {
                        state, cursor_pos, ..
                    }) = self.modal.as_mut()
                    {
                        let field_id = match state.form.fields.get(state.focused_field) {
                            Some(f)
                                if matches!(
                                    f.kind,
                                    crate::tui::editform::FieldKind::Textarea { .. }
                                ) =>
                            {
                                Some(f.id)
                            }
                            _ => None,
                        };
                        if let Some(field_id) = field_id {
                            let wrap =
                                textarea_wrap_width(&self.form_textarea_hits, state.focused_field);
                            *cursor_pos = crate::tui::form_textarea::textarea_move_cursor_vertical(
                                state.get(field_id),
                                *cursor_pos,
                                delta,
                                wrap,
                            );
                        }
                    }
                    return Some(ModalResult::Continue);
                }
                if let Some(Modal::FormEdit { scroll_offset, .. }) = self.modal.as_mut() {
                    if delta < 0 {
                        *scroll_offset = scroll_offset.saturating_sub(delta.unsigned_abs() as u16);
                    } else {
                        *scroll_offset = scroll_offset.saturating_add(delta as u16);
                    }
                }
            }
            MouseEventKind::Down(MouseButton::Left) => {
                let (x, y) = (m.column, m.row);
                let checkbox_hit = self
                    .form_checkbox_hits
                    .borrow()
                    .iter()
                    .find(|(r, _, _)| contains_rect(*r, x, y))
                    .map(|(_, field_idx, option_idx)| (*field_idx, *option_idx));
                if let Some((field_idx, option_idx)) = checkbox_hit {
                    if let Some(Modal::FormEdit { state, .. }) = self.modal.as_mut() {
                        state.focused_field = field_idx;
                        state.toggle_checkbox_at(option_idx);
                    }
                    return Some(ModalResult::Continue);
                }
                let expand_hit = self
                    .form_expand_hits
                    .borrow()
                    .iter()
                    .find(|(_, r)| contains_rect(*r, x, y))
                    .map(|(idx, _)| *idx);
                if let Some(idx) = expand_hit {
                    if let Some(Modal::FormEdit {
                        state, cursor_pos, ..
                    }) = self.modal.as_mut()
                    {
                        state.focused_field = idx;
                        let field_id = state.form.fields.get(idx).map(|f| f.id);
                        if let Some(field_id) = field_id {
                            *cursor_pos = state.get(field_id).chars().count();
                        }
                    }
                    self.form_textarea_expanded = true;
                    return Some(ModalResult::Continue);
                }
                let textarea_hit = self
                    .form_textarea_hits
                    .borrow()
                    .iter()
                    .find(|h| contains_rect(h.rect, x, y))
                    .copied();
                if let Some(hit) = textarea_hit {
                    if let Some(Modal::FormEdit {
                        state, cursor_pos, ..
                    }) = self.modal.as_mut()
                    {
                        state.focused_field = hit.field_idx;
                        let value = state.get(state.form.fields[hit.field_idx].id);
                        let local_x = x.saturating_sub(hit.rect.x);
                        let local_y = y.saturating_sub(hit.rect.y);
                        *cursor_pos = crate::tui::form_textarea::textarea_cursor_from_click(
                            value,
                            Some(hit.wrap_width),
                            hit.first_visible_row,
                            local_x,
                            local_y,
                        );
                    }
                    return Some(ModalResult::Continue);
                }
                if self.form_textarea_expanded {
                    return Some(ModalResult::Continue);
                }
                if let Some((_, idx)) = self
                    .form_field_areas
                    .borrow()
                    .iter()
                    .find(|(r, _)| contains_rect(*r, x, y))
                    .copied()
                {
                    if let Some(Modal::FormEdit {
                        state, cursor_pos, ..
                    }) = self.modal.as_mut()
                    {
                        state.focused_field = idx;
                        state.checkbox_cursor = 0;
                        *cursor_pos = state.get(state.form.fields[idx].id).chars().count();
                    }
                }
            }
            _ => {}
        }
        Some(ModalResult::Continue)
    }
}

fn textarea_wrap_width(
    hits: &std::cell::RefCell<Vec<crate::tui::form_textarea::TextareaHit>>,
    field_idx: usize,
) -> Option<u16> {
    hits.borrow()
        .iter()
        .find(|h| h.field_idx == field_idx)
        .map(|h| h.wrap_width)
        .filter(|&w| w > 0)
}

fn contains_rect(area: Rect, x: u16, y: u16) -> bool {
    x >= area.x
        && x < area.x.saturating_add(area.width)
        && y >= area.y
        && y < area.y.saturating_add(area.height)
}
