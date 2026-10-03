//! New / Open session switches and F3 jump-to-node.
use std::path::PathBuf;

use crate::starters::{self, StarterKind};
use crate::storage;
use crate::validate::{NodeStep, ValidateIssue, ValidateLoc};

use super::toasts::ToastLevel;
use super::tree::{Step, TreeId};
use super::{App, ConfirmKind, EMPTY_TEMPLATE_HINT, Modal};

impl App {
    pub(in crate::tui) fn request_new(&mut self) {
        if self.dirty {
            self.modal = Some(Modal::ConfirmPrompt {
                message: "Unsaved changes. Create a new template anyway?".to_string(),
                on_confirm: ConfirmKind::NewUnsaved,
            });
            return;
        }
        self.open_new_prompt();
    }

    pub(in crate::tui) fn request_open(&mut self) {
        if self.dirty {
            self.modal = Some(Modal::ConfirmPrompt {
                message: "Unsaved changes. Open another template anyway?".to_string(),
                on_confirm: ConfirmKind::OpenUnsaved,
            });
            return;
        }
        self.open_open_prompt();
    }

    pub(in crate::tui) fn open_new_prompt(&mut self) {
        let starter = StarterKind::Welcome;
        self.modal = Some(Modal::NewPrompt {
            path: format!("./{}", starter.as_str()),
            starter,
        });
    }

    pub(in crate::tui) fn open_open_prompt(&mut self) {
        let path = self
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        self.modal = Some(Modal::OpenPrompt { path });
    }

    pub(in crate::tui) fn submit_new(&mut self, path: String, starter: StarterKind) {
        let raw = path.trim();
        if raw.is_empty() {
            self.push_toast(ToastLevel::Warning, "New path cannot be empty.");
            self.modal = Some(Modal::NewPrompt { path, starter });
            return;
        }
        let dir = PathBuf::from(raw);
        match starters::init_template_dir(&dir, starter) {
            Ok(json) => match storage::load_template(&json) {
                Ok(template) => {
                    self.adopt_template(template, json);
                    self.push_toast(ToastLevel::Success, format!("Created {}", dir.display()));
                }
                Err(e) => {
                    self.push_toast(
                        ToastLevel::Error,
                        format!("Created but failed to load: {e}"),
                    );
                    self.modal = Some(Modal::NewPrompt { path, starter });
                }
            },
            Err(e) => {
                self.push_toast(ToastLevel::Warning, format!("{e:#}"));
                self.modal = Some(Modal::NewPrompt { path, starter });
            }
        }
    }

    pub(in crate::tui) fn submit_open(&mut self, path: String) {
        let raw = path.trim();
        if raw.is_empty() {
            self.push_toast(ToastLevel::Warning, "Open path cannot be empty.");
            self.modal = Some(Modal::OpenPrompt { path });
            return;
        }
        let p = PathBuf::from(raw);
        match storage::resolve_template_path(&p) {
            Ok(json) => match storage::load_template(&json) {
                Ok(template) => {
                    let shown = json.clone();
                    self.adopt_template(template, json);
                    self.push_toast(ToastLevel::Success, format!("Opened {}", shown.display()));
                }
                Err(e) => {
                    self.push_toast(ToastLevel::Error, format!("Failed to load: {e}"));
                    self.modal = Some(Modal::OpenPrompt { path });
                }
            },
            Err(e) => {
                self.push_toast(ToastLevel::Warning, format!("{e:#}"));
                self.modal = Some(Modal::OpenPrompt { path });
            }
        }
    }

    fn adopt_template(&mut self, template: crate::model::Template, path: PathBuf) {
        self.last_saved_json = serde_json::to_string(&template).unwrap_or_default();
        self.template = Some(template);
        storage::remember_last_template(&path);
        self.path = Some(path);
        self.dirty = false;
        self.dirty_since = None;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.collapsed.clear();
        self.selected_row = 0;
        self.tree_scroll = 0;
        self.details_scroll = 0;
        self.modal = None;
        self.form_textarea_expanded = false;
        self.paused_form_edit_modal = None;
        self.details_sync_id = None;
    }

    pub(in crate::tui) fn jump_to_issue(&mut self, issue: &ValidateIssue) {
        let Some(loc) = issue.loc.as_ref() else {
            self.modal = None;
            return;
        };
        let id = loc_to_tree_id(loc);
        super::insert::expand_ancestors(&mut self.collapsed, &id);
        self.select_tree_id(&id);
        self.open_form_edit(id, issue.field.as_deref());
    }

    pub(in crate::tui) fn open_form_edit(&mut self, id: TreeId, focus_field: Option<&str>) {
        let Some(template) = self.template.as_ref() else {
            self.push_toast(ToastLevel::Warning, EMPTY_TEMPLATE_HINT);
            return;
        };
        let Some(mut state) = crate::tui::cursor::form_for(template, &id) else {
            self.push_toast(ToastLevel::Warning, "Nothing to edit here.");
            self.modal = None;
            return;
        };
        if let Some(field) = focus_field {
            state.focus_id(field);
        }
        let cursor_pos = state
            .form
            .fields
            .get(state.focused_field)
            .map(|f| state.get(f.id).chars().count())
            .unwrap_or(0);
        self.form_textarea_expanded = false;
        self.modal = Some(Modal::FormEdit {
            cursor: id,
            state,
            cursor_pos,
            scroll_offset: 0,
            drill_stack: Vec::new(),
        });
    }
}

fn loc_to_tree_id(loc: &ValidateLoc) -> TreeId {
    match loc {
        ValidateLoc::Head => TreeId::Head,
        ValidateLoc::Brand => TreeId::Brand,
        ValidateLoc::Body => TreeId::Body,
        ValidateLoc::Node(steps) => {
            TreeId::Path(steps.iter().copied().map(node_step_to_step).collect())
        }
    }
}

fn node_step_to_step(step: NodeStep) -> Step {
    match step {
        NodeStep::BodyNode(i) => Step::BodyNode(i),
        NodeStep::WrapperChild(i) => Step::WrapperChild(i),
        NodeStep::SectionChild(i) => Step::SectionChild(i),
        NodeStep::GroupCol(i) => Step::GroupCol(i),
        NodeStep::ColComp(i) => Step::ColComp(i),
        NodeStep::HeroChild(i) => Step::HeroChild(i),
        NodeStep::NavbarLink(i) => Step::NavbarLink(i),
        NodeStep::AccordionEl(i) => Step::AccordionEl(i),
        NodeStep::CarouselImg(i) => Step::CarouselImg(i),
    }
}
