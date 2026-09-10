//! FormEdit data model: one `EditForm` per node kind, live `EditFormState`.
use std::collections::HashMap;

mod forms;
pub use forms::*;

#[derive(Debug)]
pub struct EditForm {
    pub title: &'static str,
    pub fields: &'static [FormField],
}

#[derive(Debug)]
pub struct FormField {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: FieldKind,
    #[allow(dead_code)]
    pub required: bool,
    pub visible_when: Option<FieldPredicate>,
    pub hint: Option<&'static str>,
    pub placeholder: Option<&'static str>,
}

#[derive(Debug)]
pub enum FieldKind {
    Text {
        default: &'static str,
    },
    Textarea {
        rows: u16,
        default: &'static str,
    },
    Url {
        default: &'static str,
    },
    Enum {
        options: &'static [&'static str],
        default: &'static str,
    },
    /// Compact checkbox row. `options` is `(id, label)`. Value is a
    /// comma-separated list of checked ids (border sides use `"all"`).
    Checkboxes {
        options: &'static [(&'static str, &'static str)],
        default: &'static str,
    },
    SubForm {
        template: &'static EditForm,
        min_items: usize,
        summary_field_id: &'static str,
    },
}

#[derive(Debug)]
pub enum FieldPredicate {
    FieldEquals {
        other_id: &'static str,
        value: &'static str,
    },
}

#[derive(Debug, Clone)]
pub struct EditFormState {
    pub form: &'static EditForm,
    pub values: HashMap<String, String>,
    pub sub_state: HashMap<String, Vec<EditFormState>>,
    pub selected_sub_item: HashMap<String, usize>,
    pub focused_field: usize,
    pub textarea_cursor: (usize, usize),
    /// Index of the focused checkbox inside a `Checkboxes` field.
    pub checkbox_cursor: usize,
}

impl EditFormState {
    pub fn new(form: &'static EditForm) -> Self {
        let mut values = HashMap::new();
        let mut sub_state = HashMap::new();
        let mut selected_sub_item = HashMap::new();
        for field in form.fields {
            match &field.kind {
                FieldKind::Text { default } | FieldKind::Url { default } => {
                    values.insert(field.id.to_string(), default.to_string());
                }
                FieldKind::Textarea { default, .. } => {
                    values.insert(field.id.to_string(), default.to_string());
                }
                FieldKind::Enum { default, .. } => {
                    values.insert(field.id.to_string(), default.to_string());
                }
                FieldKind::Checkboxes { default, .. } => {
                    values.insert(field.id.to_string(), default.to_string());
                }
                FieldKind::SubForm { .. } => {
                    sub_state.insert(field.id.to_string(), Vec::new());
                    selected_sub_item.insert(field.id.to_string(), 0);
                }
            }
        }
        Self {
            form,
            values,
            sub_state,
            selected_sub_item,
            focused_field: 0,
            textarea_cursor: (0, 0),
            checkbox_cursor: 0,
        }
    }

    pub fn new_sub_item(&self, subform_field_id: &str) -> Option<EditFormState> {
        for field in self.form.fields {
            if field.id == subform_field_id {
                if let FieldKind::SubForm { template, .. } = &field.kind {
                    return Some(EditFormState::new(*template));
                }
            }
        }
        None
    }

    pub fn get(&self, id: &str) -> &str {
        self.values.get(id).map(String::as_str).unwrap_or("")
    }

    pub fn set(&mut self, id: &str, value: impl Into<String>) {
        self.values.insert(id.to_string(), value.into());
    }

    pub fn field_visible(&self, field: &FormField) -> bool {
        match &field.visible_when {
            None => true,
            Some(FieldPredicate::FieldEquals { other_id, value }) => self.get(other_id) == *value,
        }
    }

    pub fn visible_field_indices(&self) -> Vec<usize> {
        self.form
            .fields
            .iter()
            .enumerate()
            .filter_map(|(idx, field)| self.field_visible(field).then_some(idx))
            .collect()
    }

    pub fn focus_next(&mut self) {
        let visible = self.visible_field_indices();
        if visible.is_empty() {
            return;
        }
        let current_pos = visible
            .iter()
            .position(|&i| i == self.focused_field)
            .unwrap_or(0);
        self.focused_field = visible[(current_pos + 1) % visible.len()];
        self.textarea_cursor = (0, 0);
        self.checkbox_cursor = 0;
    }

    pub fn focus_prev(&mut self) {
        let visible = self.visible_field_indices();
        if visible.is_empty() {
            return;
        }
        let current_pos = visible
            .iter()
            .position(|&i| i == self.focused_field)
            .unwrap_or(0);
        let prev_pos = if current_pos == 0 {
            visible.len() - 1
        } else {
            current_pos - 1
        };
        self.focused_field = visible[prev_pos];
        self.textarea_cursor = (0, 0);
        self.checkbox_cursor = 0;
    }

    pub fn focused(&self) -> Option<&FormField> {
        self.form.fields.get(self.focused_field)
    }

    pub fn cycle_enum(&mut self, forward: bool) {
        let Some(field) = self.focused() else {
            return;
        };
        let FieldKind::Enum { options, .. } = &field.kind else {
            return;
        };
        if options.is_empty() {
            return;
        }
        let current = self.get(field.id).to_string();
        let idx = options
            .iter()
            .position(|opt| *opt == current.as_str())
            .unwrap_or(0);
        let next = if forward {
            (idx + 1) % options.len()
        } else if idx == 0 {
            options.len() - 1
        } else {
            idx - 1
        };
        self.set(field.id, options[next].to_string());
    }

    pub fn move_checkbox_cursor(&mut self, delta: i32) {
        let Some(field) = self.focused() else {
            return;
        };
        let FieldKind::Checkboxes { options, .. } = &field.kind else {
            return;
        };
        let n = options.len() as i32;
        if n == 0 {
            return;
        }
        self.checkbox_cursor = (self.checkbox_cursor as i32 + delta).rem_euclid(n) as usize;
    }

    pub fn toggle_focused_checkbox(&mut self) {
        let Some(field) = self.focused() else {
            return;
        };
        let FieldKind::Checkboxes { options, .. } = &field.kind else {
            return;
        };
        let Some(&(id, _)) = options.get(self.checkbox_cursor) else {
            return;
        };
        let current = self.get(field.id).to_string();
        let next = crate::border::Sides::from_storage(&current)
            .unwrap_or(crate::border::Sides::ALL)
            .toggle(id);
        let field_id = field.id.to_string();
        self.set(&field_id, next.to_form());
    }

    pub fn toggle_checkbox_at(&mut self, option_idx: usize) {
        self.checkbox_cursor = option_idx;
        self.toggle_focused_checkbox();
    }

    #[cfg(test)]
    pub fn field_index(&self, id: &str) -> Option<usize> {
        self.form.fields.iter().position(|f| f.id == id)
    }
}

/// Hit rects for each checkbox inside `origin`, left-to-right with a 2-col gap.
pub fn checkbox_item_rects(
    options: &[(&str, &str)],
    origin: ratatui::layout::Rect,
) -> Vec<ratatui::layout::Rect> {
    let mut x = origin.x;
    let mut out = Vec::with_capacity(options.len());
    for (i, &(_, label)) in options.iter().enumerate() {
        if i > 0 {
            x = x.saturating_add(2);
        }
        let w = 4u16.saturating_add(label.len() as u16);
        let used = x.saturating_sub(origin.x);
        if used >= origin.width {
            break;
        }
        let width = w.min(origin.width.saturating_sub(used));
        out.push(ratatui::layout::Rect {
            x,
            y: origin.y,
            width,
            height: 1,
        });
        x = x.saturating_add(w);
    }
    out
}
