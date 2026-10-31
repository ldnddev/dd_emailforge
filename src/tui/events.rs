//! Keyboard and mouse dispatch for chrome (F1/F2/F3/s/Ctrl+Q).
use super::*;

/// Inputs that cannot change the frame. Mouse capture streams `Moved`/`Drag`
/// continuously; Release fires on Kitty/Ghostty/WezTerm/Windows.
/// `FocusGained` is not idle: a followed Omarchy theme may need a refresh.
fn is_idle_input(evt: &Event) -> bool {
    match evt {
        Event::FocusLost => true,
        Event::Key(k) => matches!(k.kind, KeyEventKind::Release),
        Event::Mouse(m) => !matches!(
            m.kind,
            MouseEventKind::Down(_) | MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
        ),
        _ => false,
    }
}

impl App {
    /// Returns whether the frame changed and needs a redraw.
    pub(super) fn handle_event(&mut self, evt: Event) -> anyhow::Result<bool> {
        if matches!(evt, Event::Resize(_, _)) {
            return Ok(true);
        }
        if matches!(evt, Event::FocusGained) {
            return Ok(self.refresh_followed_theme());
        }
        if is_idle_input(&evt) {
            return Ok(false);
        }

        if self.show_help {
            return Ok(self.handle_help_event(evt));
        }

        if self.show_theme {
            return Ok(self.handle_theme_event(evt));
        }

        if self.modal.is_some() {
            // Only FormEdit consumes mouse; other dialogs ignore it.
            if matches!(evt, Event::Mouse(_)) && !matches!(self.modal, Some(Modal::FormEdit { .. }))
            {
                return Ok(false);
            }
            let _ = self.handle_modal_event(evt);
            return Ok(true);
        }

        let changed = match evt {
            Event::Key(k) => match k.code {
                KeyCode::F(1) => {
                    self.show_help = true;
                    true
                }
                KeyCode::F(2) => {
                    self.open_theme_ui();
                    true
                }
                KeyCode::F(3) => {
                    self.open_validation_modal();
                    true
                }
                KeyCode::Char('n') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.request_new();
                    true
                }
                KeyCode::Char('o') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.request_open();
                    true
                }
                KeyCode::Char('p') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.begin_preview();
                    true
                }
                KeyCode::Char('E') if k.modifiers.contains(KeyModifiers::SHIFT) => {
                    self.begin_export();
                    true
                }
                KeyCode::Char('s') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.begin_save();
                    true
                }
                KeyCode::Char('q') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.request_quit();
                    true
                }
                KeyCode::Tab | KeyCode::BackTab => {
                    if self.details_visible {
                        self.pane = match self.pane {
                            super::tree::PaneFocus::Structure => super::tree::PaneFocus::Details,
                            super::tree::PaneFocus::Details => super::tree::PaneFocus::Structure,
                        };
                        true
                    } else {
                        false
                    }
                }
                KeyCode::Enter => {
                    self.tree_enter();
                    true
                }
                KeyCode::Char('/') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.open_insert_picker();
                    true
                }
                KeyCode::Char('d') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.delete_selected_row();
                    true
                }
                KeyCode::Char('y') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.duplicate_selected_row();
                    true
                }
                KeyCode::Char('u') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.undo_last();
                    true
                }
                KeyCode::Char('U') => {
                    self.redo_last();
                    true
                }
                KeyCode::Char('r') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.redo_last();
                    true
                }
                KeyCode::Char('J') => {
                    self.reorder_selected(1);
                    true
                }
                KeyCode::Char('K') => {
                    self.reorder_selected(-1);
                    true
                }
                KeyCode::Char('C') => {
                    self.add_column();
                    true
                }
                KeyCode::Char('V') => {
                    self.remove_column();
                    true
                }
                KeyCode::Char('c') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.hop_column(-1);
                    true
                }
                KeyCode::Char('v') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.hop_column(1);
                    true
                }
                _ => self.handle_tree_nav(k),
            },
            Event::Mouse(m) => self.handle_tree_mouse(m),
            _ => false,
        };

        Ok(changed)
    }

    fn handle_help_event(&mut self, evt: Event) -> bool {
        let before = (self.show_help, self.help_scroll);
        match evt {
            Event::Key(k) => match k.code {
                KeyCode::F(1) | KeyCode::Esc => {
                    self.show_help = false;
                    self.help_scroll = 0;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.help_scroll = self.help_scroll.saturating_add(1).min(self.help_scroll_max);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.help_scroll = self.help_scroll.saturating_sub(1);
                }
                KeyCode::PageDown => {
                    self.help_scroll = self
                        .help_scroll
                        .saturating_add(10)
                        .min(self.help_scroll_max);
                }
                KeyCode::PageUp => {
                    self.help_scroll = self.help_scroll.saturating_sub(10);
                }
                KeyCode::Home | KeyCode::Char('g') => {
                    self.help_scroll = 0;
                }
                KeyCode::End | KeyCode::Char('G') => {
                    self.help_scroll = self.help_scroll_max;
                }
                _ => {}
            },
            Event::Mouse(m) => match m.kind {
                MouseEventKind::ScrollUp => {
                    self.help_scroll = self.help_scroll.saturating_sub(3);
                }
                MouseEventKind::ScrollDown => {
                    self.help_scroll = self.help_scroll.saturating_add(3).min(self.help_scroll_max);
                }
                _ => {}
            },
            _ => {}
        }
        (self.show_help, self.help_scroll) != before
    }

    fn handle_theme_event(&mut self, evt: Event) -> bool {
        match evt {
            Event::Key(k) => {
                if k.code == KeyCode::F(2) {
                    self.close_theme(true);
                    true
                } else if let Some(ek) = map_editor_key(k) {
                    let shift = k.modifiers.contains(KeyModifiers::SHIFT);
                    if self.theme_editor.is_some() {
                        self.dispatch_theme_editor(ek, shift);
                    } else {
                        self.dispatch_theme_picker(ek);
                    }
                    true
                } else {
                    false
                }
            }
            Event::Mouse(m) => match m.kind {
                MouseEventKind::ScrollUp => {
                    if self.theme_editor.is_some() || self.theme_picker.is_some() {
                        self.scroll_theme_selection(-1)
                    } else {
                        let before = self.theme_scroll;
                        self.theme_scroll = self.theme_scroll.saturating_sub(3);
                        self.theme_scroll != before
                    }
                }
                MouseEventKind::ScrollDown => {
                    if self.theme_editor.is_some() || self.theme_picker.is_some() {
                        self.scroll_theme_selection(1)
                    } else {
                        let before = self.theme_scroll;
                        self.theme_scroll = self
                            .theme_scroll
                            .saturating_add(3)
                            .min(self.theme_scroll_max);
                        self.theme_scroll != before
                    }
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Re-read `omarchy: follow` when the desktop theme file changes.
    /// Does not reshuffle `header_copy`.
    fn refresh_followed_theme(&mut self) -> bool {
        if self.show_theme {
            return false;
        }
        if !matches!(
            self.theme.omarchy,
            Some(ldnddev_theme::OmarchyChoice::Follow)
        ) {
            return false;
        }
        let stamp = ldnddev_theme::follow_stamp(&self.omarchy_paths);
        if stamp == self.follow_stamp {
            return false;
        }
        let (theme, source, warning) =
            AppTheme::load_resolved(crate::paths::theme_candidates(), &self.omarchy_paths);
        self.follow_stamp = stamp;
        self.theme = theme;
        self.theme_source = source;
        self.theme_status = warning.clone();
        if let Some(msg) = warning {
            self.push_toast(ToastLevel::Warning, msg);
        }
        true
    }

    fn open_theme_ui(&mut self) {
        self.show_theme = true;
        let palette = super::theme::palette_from_theme(&self.theme);
        let picker = ldnddev_theme::ThemePicker::new(
            palette.clone(),
            super::theme::extra_theme_fields(),
            self.omarchy_paths.clone(),
        );
        if picker.lists_omarchy() {
            self.theme_picker = Some(picker);
            self.theme_editor = None;
        } else {
            self.theme_picker = None;
            self.theme_editor = Some(ldnddev_theme::ThemeEditor::new(
                palette,
                super::theme::extra_theme_fields(),
            ));
        }
    }

    fn scroll_theme_selection(&mut self, delta: i32) -> bool {
        if let Some(editor) = self.theme_editor.as_mut() {
            let before = editor.selected;
            let next = editor.selected as i32 + delta;
            editor.select(next.max(0) as usize);
            return editor.selected != before;
        }
        if let Some(picker) = self.theme_picker.as_mut() {
            let before = picker.selected;
            let next = picker.selected as i32 + delta;
            picker.select(next.max(0) as usize);
            return picker.selected != before;
        }
        false
    }

    fn dispatch_theme_picker(&mut self, ek: ldnddev_theme::EditorKey) {
        let outcome = {
            let Some(picker) = self.theme_picker.as_mut() else {
                return;
            };
            picker.handle(ek)
        };
        match outcome {
            ldnddev_theme::PickerOutcome::Applied => {
                if let Some(picker) = &self.theme_picker {
                    super::theme::apply_palette(&mut self.theme, &picker.palette);
                }
            }
            ldnddev_theme::PickerOutcome::EditColors => {
                let Some(picker) = self.theme_picker.as_ref() else {
                    return;
                };
                let mut editor = ldnddev_theme::ThemeEditor::new(
                    picker.palette.clone(),
                    super::theme::extra_theme_fields(),
                );
                editor.save_target = picker.save_target;
                self.theme_editor = Some(editor);
            }
            ldnddev_theme::PickerOutcome::RequestSave => {
                let saved = self
                    .theme_picker
                    .as_ref()
                    .map(|picker| (picker.palette.clone(), picker.save_target));
                if let Some((palette, target)) = saved {
                    self.save_theme_palette(&palette, target);
                }
            }
            ldnddev_theme::PickerOutcome::Error(msg) => {
                self.push_toast(ToastLevel::Error, msg);
            }
            ldnddev_theme::PickerOutcome::Closed { .. } => self.close_theme(true),
            ldnddev_theme::PickerOutcome::None => {}
        }
    }

    fn save_theme_palette(
        &mut self,
        palette: &ldnddev_theme::Palette,
        target: ldnddev_theme::ThemeSaveTarget,
    ) {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let fields = self
            .theme_editor
            .as_ref()
            .map(|editor| editor.fields.clone())
            .unwrap_or_else(|| {
                let mut fields = ldnddev_theme::COLOR_FIELDS.to_vec();
                for extra in super::theme::extra_theme_fields() {
                    if !fields.iter().any(|field| field.key == extra.key) {
                        fields.push(*extra);
                    }
                }
                fields
            });
        match ldnddev_theme::save_theme(
            palette,
            &root,
            "dd_emailforge_theme.yml",
            target,
            ldnddev_theme::default_config_home().as_deref(),
            &fields,
        ) {
            Ok(path) => {
                super::theme::apply_palette(&mut self.theme, palette);
                self.theme_source = target.label().to_string();
                self.follow_stamp = ldnddev_theme::follow_stamp(&self.omarchy_paths);
                self.theme_editor = None;
                self.theme_picker = None;
                self.show_theme = false;
                self.theme_scroll = 0;
                self.push_toast(
                    ToastLevel::Success,
                    format!("Theme saved ({})", path.display()),
                );
            }
            Err(err) => {
                self.push_toast(ToastLevel::Error, format!("Theme save failed: {err}"));
            }
        }
    }

    fn close_theme(&mut self, revert: bool) {
        if revert {
            if let Some(picker) = self.theme_picker.as_ref() {
                let palette = picker.revert_palette();
                super::theme::apply_palette(&mut self.theme, &palette);
            } else if let Some(mut editor) = self.theme_editor.take() {
                editor.revert();
                super::theme::apply_palette(&mut self.theme, &editor.palette);
            }
        }
        self.theme_picker = None;
        self.theme_editor = None;
        self.show_theme = false;
        self.theme_scroll = 0;
    }

    fn handle_tree_nav(&mut self, k: event::KeyEvent) -> bool {
        use super::tree::PaneFocus;
        let details = self.pane == PaneFocus::Details && self.details_visible;
        let before = (
            self.selected_row,
            self.details_scroll,
            self.collapsed.clone(),
        );
        match k.code {
            KeyCode::Down | KeyCode::Char('j') if !details => self.tree_move(1),
            KeyCode::Up | KeyCode::Char('k') if !details => self.tree_move(-1),
            KeyCode::Down | KeyCode::Char('j') if details => {
                self.details_scroll = (self.details_scroll + 1).min(self.details_scroll_max);
            }
            KeyCode::Up | KeyCode::Char('k') if details => {
                self.details_scroll = self.details_scroll.saturating_sub(1);
            }
            KeyCode::PageDown if details => {
                self.details_scroll = (self.details_scroll + 10).min(self.details_scroll_max);
            }
            KeyCode::PageUp if details => {
                self.details_scroll = self.details_scroll.saturating_sub(10);
            }
            KeyCode::PageDown => self.tree_move(10),
            KeyCode::PageUp => self.tree_move(-10),
            KeyCode::Home | KeyCode::Char('g') if !k.modifiers.contains(KeyModifiers::SHIFT) => {
                if details {
                    self.details_scroll = 0;
                } else {
                    self.tree_home();
                }
            }
            KeyCode::End | KeyCode::Char('G') => {
                if details {
                    self.details_scroll = self.details_scroll_max;
                } else {
                    self.tree_end();
                }
            }
            KeyCode::Left | KeyCode::Char('h')
                if !k.modifiers.contains(KeyModifiers::CONTROL) && !details =>
            {
                self.tree_collapse();
            }
            KeyCode::Right | KeyCode::Char('l')
                if !k.modifiers.contains(KeyModifiers::CONTROL) && !details =>
            {
                self.tree_expand();
            }
            KeyCode::Char(' ') if !details => self.tree_toggle_expand(),
            _ => {}
        }
        (
            self.selected_row,
            self.details_scroll,
            self.collapsed.clone(),
        ) != before
    }

    fn handle_tree_mouse(&mut self, m: event::MouseEvent) -> bool {
        use super::tree::PaneFocus;
        let (x, y) = (m.column, m.row);
        let in_tree = contains(self.tree_area, x, y);
        let in_details = self.details_visible && contains(self.details_area, x, y);
        let before = (
            self.selected_row,
            self.tree_scroll,
            self.details_scroll,
            self.pane,
            self.collapsed.clone(),
            self.modal.is_some(),
            self.toasts.len(),
        );
        match m.kind {
            MouseEventKind::ScrollUp if in_details => {
                self.details_scroll = self.details_scroll.saturating_sub(3);
            }
            MouseEventKind::ScrollDown if in_details => {
                self.details_scroll = (self.details_scroll + 3).min(self.details_scroll_max);
            }
            MouseEventKind::ScrollUp if in_tree => {
                self.tree_scroll = self.tree_scroll.saturating_sub(3);
            }
            MouseEventKind::ScrollDown if in_tree => {
                let visible = self.tree_area.height.saturating_sub(2) as usize;
                let max_scroll = self.tree_rows().len().saturating_sub(visible);
                self.tree_scroll = (self.tree_scroll + 3).min(max_scroll);
            }
            MouseEventKind::Down(MouseButton::Left) if in_tree => {
                self.pane = PaneFocus::Structure;
                let inner_y = self.tree_area.y.saturating_add(1);
                if y >= inner_y {
                    let idx = self.tree_scroll + (y - inner_y) as usize;
                    let n = self.tree_rows().len();
                    if idx < n {
                        self.selected_row = idx;
                        let glyph = x < self.tree_area.x.saturating_add(6);
                        if glyph {
                            self.tree_toggle_expand();
                        }
                        let now = std::time::Instant::now();
                        if let Some((px, py, t0)) = self.last_click {
                            if px == x
                                && py == y
                                && now.duration_since(t0).as_millis()
                                    < super::DOUBLE_CLICK_THRESHOLD_MS
                            {
                                self.tree_enter();
                            }
                        }
                        self.last_click = Some((x, y, now));
                    }
                }
            }
            MouseEventKind::Down(MouseButton::Left) if in_details => {
                self.pane = PaneFocus::Details;
                if let Some((_, id)) = self
                    .details_hit_areas
                    .iter()
                    .rev()
                    .find(|(r, _)| contains(*r, x, y))
                    .cloned()
                {
                    self.collapsed.remove(&super::tree::TreeId::Body);
                    if let super::tree::TreeId::Path(path) = &id {
                        for i in 1..path.len() {
                            self.collapsed
                                .remove(&super::tree::TreeId::Path(path[..i].to_vec()));
                        }
                    }
                    self.select_tree_id(&id);
                }
            }
            _ => {}
        }
        (
            self.selected_row,
            self.tree_scroll,
            self.details_scroll,
            self.pane,
            self.collapsed.clone(),
            self.modal.is_some(),
            self.toasts.len(),
        ) != before
    }

    fn dispatch_theme_editor(&mut self, ek: ldnddev_theme::EditorKey, shift: bool) {
        let outcome = {
            let Some(editor) = self.theme_editor.as_mut() else {
                return;
            };
            editor.handle(ek, shift)
        };
        match outcome {
            ldnddev_theme::EditorOutcome::PaletteChanged => {
                if let Some(editor) = &self.theme_editor {
                    if let Some(picker) = self.theme_picker.as_mut() {
                        picker.palette = editor.palette.clone();
                        picker.save_target = editor.save_target;
                    }
                    super::theme::apply_palette(&mut self.theme, &editor.palette);
                }
            }
            ldnddev_theme::EditorOutcome::RequestSave => {
                let saved = self
                    .theme_editor
                    .as_ref()
                    .map(|editor| (editor.palette.clone(), editor.save_target));
                if let Some((palette, target)) = saved {
                    self.save_theme_palette(&palette, target);
                }
            }
            ldnddev_theme::EditorOutcome::Closed { .. } => {
                if self.theme_picker.is_some() {
                    if let Some(editor) = self.theme_editor.take() {
                        if let Some(picker) = self.theme_picker.as_mut() {
                            picker.palette = editor.palette.clone();
                            picker.save_target = editor.save_target;
                        }
                        super::theme::apply_palette(&mut self.theme, &editor.palette);
                    }
                } else {
                    self.close_theme(true);
                }
            }
            ldnddev_theme::EditorOutcome::HexError(msg) => {
                self.push_toast(ToastLevel::Error, msg);
            }
            ldnddev_theme::EditorOutcome::None => {}
        }
    }
}

fn map_editor_key(k: crossterm::event::KeyEvent) -> Option<ldnddev_theme::EditorKey> {
    Some(match k.code {
        KeyCode::Up => ldnddev_theme::EditorKey::Up,
        KeyCode::Down => ldnddev_theme::EditorKey::Down,
        KeyCode::Left => ldnddev_theme::EditorKey::Left,
        KeyCode::Right => ldnddev_theme::EditorKey::Right,
        KeyCode::Tab => ldnddev_theme::EditorKey::Tab,
        KeyCode::Enter => ldnddev_theme::EditorKey::Enter,
        KeyCode::Esc => ldnddev_theme::EditorKey::Esc,
        KeyCode::Backspace => ldnddev_theme::EditorKey::Backspace,
        KeyCode::Char(c) => ldnddev_theme::EditorKey::Char(c),
        _ => return None,
    })
}

fn contains(area: Rect, x: u16, y: u16) -> bool {
    x >= area.x
        && x < area.x.saturating_add(area.width)
        && y >= area.y
        && y < area.y.saturating_add(area.height)
}
