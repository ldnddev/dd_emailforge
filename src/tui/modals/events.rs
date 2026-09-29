use super::super::*;

impl App {
    pub(in crate::tui) fn handle_modal_event(&mut self, evt: Event) -> Option<ModalResult> {
        let _ = self.modal.as_ref()?;

        if let Event::Mouse(m) = &evt {
            if matches!(self.modal, Some(Modal::FormEdit { .. })) {
                return self.handle_form_edit_mouse(*m);
            }
            return Some(ModalResult::Continue);
        }

        if let Event::Paste(text) = &evt {
            return self.handle_modal_paste(text);
        }

        if let Event::Key(key) = &evt {
            if key.code == KeyCode::F(1) {
                self.show_help = true;
                self.help_scroll = 0;
                return Some(ModalResult::Continue);
            }
            if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
                self.request_quit();
                return Some(ModalResult::Continue);
            }
            let key = *key;
            return match self.modal.as_ref()? {
                Modal::LoadError { .. } => self.handle_load_error_event(key),
                Modal::SavePrompt { .. } => self.handle_save_prompt_event(key),
                Modal::OpenPrompt { .. } => self.handle_open_prompt_event(key),
                Modal::NewPrompt { .. } => self.handle_new_prompt_event(key),
                Modal::ConfirmPrompt { .. } => self.handle_confirm_prompt_event(key),
                Modal::ValidationErrors { .. } => self.handle_validation_errors_event(key),
                Modal::MjmlMissing { .. } => self.handle_load_error_event(key),
                Modal::MjmlCompileError { .. } => self.handle_compile_error_event(key),
                Modal::FormEdit { .. } => self.handle_form_edit_event(key),
                Modal::ComponentPicker { .. } => self.handle_component_picker_event(key),
                Modal::ImagePicker { .. } => self.handle_image_picker_event(key),
            };
        }
        Some(ModalResult::Continue)
    }

    fn handle_modal_paste(&mut self, text: &str) -> Option<ModalResult> {
        match self.modal.as_mut() {
            Some(Modal::SavePrompt { path }) => {
                path.push_str(&crate::tui::form_textarea::sanitize_paste(text, false));
                Some(ModalResult::Continue)
            }
            Some(Modal::OpenPrompt { path }) => {
                path.push_str(&crate::tui::form_textarea::sanitize_paste(text, false));
                Some(ModalResult::Continue)
            }
            Some(Modal::NewPrompt { path, .. }) => {
                path.push_str(&crate::tui::form_textarea::sanitize_paste(text, false));
                Some(ModalResult::Continue)
            }
            Some(Modal::FormEdit { .. }) => self.paste_into_form_edit(text),
            Some(Modal::ComponentPicker { query, selected }) => {
                query.push_str(&crate::tui::form_textarea::sanitize_paste(text, false));
                *selected = 0;
                Some(ModalResult::Continue)
            }
            Some(Modal::ImagePicker { state }) => {
                state
                    .filter
                    .push_str(&crate::tui::form_textarea::sanitize_paste(text, false));
                state.selected = 0;
                Some(ModalResult::Continue)
            }
            _ => Some(ModalResult::Continue),
        }
    }

    fn handle_load_error_event(&mut self, key: event::KeyEvent) -> Option<ModalResult> {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => {
                self.modal = None;
                Some(ModalResult::CloseSuccess)
            }
            _ => Some(ModalResult::Continue),
        }
    }

    fn handle_save_prompt_event(&mut self, key: event::KeyEvent) -> Option<ModalResult> {
        let path = if let Some(Modal::SavePrompt { path }) = self.modal.take() {
            path
        } else {
            return Some(ModalResult::CloseCancel);
        };

        match key.code {
            KeyCode::Esc => {
                self.push_toast(ToastLevel::Info, "Save cancelled.");
                Some(ModalResult::CloseCancel)
            }
            KeyCode::Enter | KeyCode::Char('s')
                if key.modifiers.contains(KeyModifiers::CONTROL) || key.code == KeyCode::Enter =>
            {
                let raw = path.trim();
                if raw.is_empty() {
                    self.push_toast(ToastLevel::Warning, "Save path cannot be empty.");
                    self.modal = Some(Modal::SavePrompt { path });
                    Some(ModalResult::Continue)
                } else {
                    let path_buf = std::path::PathBuf::from(raw);
                    if let Err(e) = self.commit_save_with_backup(&path_buf) {
                        self.push_toast(ToastLevel::Error, format!("Failed to save: {e}"));
                        self.modal = Some(Modal::SavePrompt { path });
                        Some(ModalResult::Continue)
                    } else {
                        self.push_toast(
                            ToastLevel::Success,
                            format!("Saved {}", path_buf.display()),
                        );
                        Some(ModalResult::CloseSuccess)
                    }
                }
            }
            KeyCode::Backspace => {
                let mut new_path = path;
                new_path.pop();
                self.modal = Some(Modal::SavePrompt { path: new_path });
                Some(ModalResult::Continue)
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                let mut new_path = path;
                new_path.push(c);
                self.modal = Some(Modal::SavePrompt { path: new_path });
                Some(ModalResult::Continue)
            }
            _ => {
                self.modal = Some(Modal::SavePrompt { path });
                Some(ModalResult::Continue)
            }
        }
    }

    fn handle_confirm_prompt_event(&mut self, key: event::KeyEvent) -> Option<ModalResult> {
        let kind = match &self.modal {
            Some(Modal::ConfirmPrompt { on_confirm, .. }) => *on_confirm,
            _ => return Some(ModalResult::CloseCancel),
        };
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                match kind {
                    ConfirmKind::QuitUnsaved => {
                        self.should_quit = true;
                        self.modal = None;
                    }
                    ConfirmKind::NewUnsaved => self.open_new_prompt(),
                    ConfirmKind::OpenUnsaved => self.open_open_prompt(),
                }
                Some(ModalResult::CloseSuccess)
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.modal = None;
                self.push_toast(ToastLevel::Info, "Cancelled.");
                Some(ModalResult::CloseCancel)
            }
            _ => Some(ModalResult::Continue),
        }
    }

    fn handle_validation_errors_event(&mut self, key: event::KeyEvent) -> Option<ModalResult> {
        let (errors_len, scroll) = match &self.modal {
            Some(Modal::ValidationErrors {
                errors,
                scroll_offset,
            }) => (errors.len(), *scroll_offset),
            _ => return Some(ModalResult::CloseCancel),
        };
        match key.code {
            KeyCode::Enter => {
                let issue = match &self.modal {
                    Some(Modal::ValidationErrors {
                        errors,
                        scroll_offset,
                    }) => errors.get(*scroll_offset).cloned(),
                    _ => None,
                };
                if let Some(issue) = issue {
                    self.jump_to_issue(&issue);
                } else {
                    self.modal = None;
                }
                Some(ModalResult::CloseSuccess)
            }
            KeyCode::Esc => {
                self.modal = None;
                Some(ModalResult::CloseSuccess)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(Modal::ValidationErrors { scroll_offset, .. }) = self.modal.as_mut() {
                    *scroll_offset = scroll_offset.saturating_sub(1);
                }
                Some(ModalResult::Continue)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(Modal::ValidationErrors { scroll_offset, .. }) = self.modal.as_mut() {
                    if scroll + 1 < errors_len.max(1) {
                        *scroll_offset += 1;
                    }
                }
                Some(ModalResult::Continue)
            }
            KeyCode::PageUp => {
                if let Some(Modal::ValidationErrors { scroll_offset, .. }) = self.modal.as_mut() {
                    *scroll_offset = scroll_offset.saturating_sub(5);
                }
                Some(ModalResult::Continue)
            }
            KeyCode::PageDown => {
                if let Some(Modal::ValidationErrors { scroll_offset, .. }) = self.modal.as_mut() {
                    *scroll_offset = (scroll + 5).min(errors_len.saturating_sub(1));
                }
                Some(ModalResult::Continue)
            }
            _ => Some(ModalResult::Continue),
        }
    }

    fn handle_compile_error_event(&mut self, key: event::KeyEvent) -> Option<ModalResult> {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => {
                self.modal = None;
                Some(ModalResult::CloseSuccess)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(Modal::MjmlCompileError { scroll, .. }) = self.modal.as_mut() {
                    *scroll = scroll.saturating_add(1);
                }
                Some(ModalResult::Continue)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(Modal::MjmlCompileError { scroll, .. }) = self.modal.as_mut() {
                    *scroll = scroll.saturating_sub(1);
                }
                Some(ModalResult::Continue)
            }
            _ => Some(ModalResult::Continue),
        }
    }

    pub(in crate::tui) fn open_validation_modal(&mut self) {
        let Some(template) = self.template.as_ref() else {
            self.push_toast(ToastLevel::Warning, super::super::EMPTY_TEMPLATE_HINT);
            return;
        };
        let root = self.path.as_ref().map(|p| crate::storage::template_root(p));
        let report = crate::validate::validate_template_with_root(template, root.as_deref());
        if !report.errors.is_empty() {
            self.modal = Some(Modal::ValidationErrors {
                errors: report.errors,
                scroll_offset: 0,
            });
            return;
        }
        self.push_toast(ToastLevel::Success, "Validation passed");
        for warn in report.warnings {
            self.push_toast(ToastLevel::Warning, warn.to_string());
        }
    }

    fn handle_open_prompt_event(&mut self, key: event::KeyEvent) -> Option<ModalResult> {
        let path = if let Some(Modal::OpenPrompt { path }) = self.modal.take() {
            path
        } else {
            return Some(ModalResult::CloseCancel);
        };
        match key.code {
            KeyCode::Esc => {
                self.push_toast(ToastLevel::Info, "Open cancelled.");
                Some(ModalResult::CloseCancel)
            }
            KeyCode::Enter => {
                self.submit_open(path);
                Some(ModalResult::Continue)
            }
            KeyCode::Backspace => {
                let mut new_path = path;
                new_path.pop();
                self.modal = Some(Modal::OpenPrompt { path: new_path });
                Some(ModalResult::Continue)
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                let mut new_path = path;
                new_path.push(c);
                self.modal = Some(Modal::OpenPrompt { path: new_path });
                Some(ModalResult::Continue)
            }
            _ => {
                self.modal = Some(Modal::OpenPrompt { path });
                Some(ModalResult::Continue)
            }
        }
    }

    fn handle_new_prompt_event(&mut self, key: event::KeyEvent) -> Option<ModalResult> {
        let (path, starter) = if let Some(Modal::NewPrompt { path, starter }) = self.modal.take() {
            (path, starter)
        } else {
            return Some(ModalResult::CloseCancel);
        };
        match key.code {
            KeyCode::Esc => {
                self.push_toast(ToastLevel::Info, "New cancelled.");
                Some(ModalResult::CloseCancel)
            }
            KeyCode::Enter => {
                self.submit_new(path, starter);
                Some(ModalResult::Continue)
            }
            KeyCode::Tab | KeyCode::BackTab => {
                let old_default = format!("./{}", starter.as_str());
                let starter = if key.code == KeyCode::BackTab {
                    starter.prev()
                } else {
                    starter.next()
                };
                let path = if path.trim() == old_default {
                    format!("./{}", starter.as_str())
                } else {
                    path
                };
                self.modal = Some(Modal::NewPrompt { path, starter });
                Some(ModalResult::Continue)
            }
            KeyCode::Backspace => {
                let mut new_path = path;
                new_path.pop();
                self.modal = Some(Modal::NewPrompt {
                    path: new_path,
                    starter,
                });
                Some(ModalResult::Continue)
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                let mut new_path = path;
                new_path.push(c);
                self.modal = Some(Modal::NewPrompt {
                    path: new_path,
                    starter,
                });
                Some(ModalResult::Continue)
            }
            _ => {
                self.modal = Some(Modal::NewPrompt { path, starter });
                Some(ModalResult::Continue)
            }
        }
    }
}
