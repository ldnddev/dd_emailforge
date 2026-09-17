//! Keyboard and mouse dispatch for chrome (F1/F2/F3/s/Ctrl+Q).
use super::*;

impl App {
    pub(super) fn handle_event(&mut self, evt: Event) -> anyhow::Result<()> {
        if self.show_help {
            match evt {
                Event::Key(k) => match k.code {
                    KeyCode::F(1) | KeyCode::Esc => {
                        self.show_help = false;
                        self.help_scroll = 0;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.help_scroll =
                            self.help_scroll.saturating_add(1).min(self.help_scroll_max);
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
                        self.help_scroll =
                            self.help_scroll.saturating_add(3).min(self.help_scroll_max);
                    }
                    _ => {}
                },
                _ => {}
            }
            return Ok(());
        }

        if self.show_theme {
            match evt {
                Event::Key(k) => {
                    if k.code == KeyCode::F(2) {
                        if let Some(mut editor) = self.theme_editor.take() {
                            editor.revert();
                            super::theme::apply_palette(&mut self.theme, &editor.palette);
                        }
                        self.show_theme = false;
                        self.theme_scroll = 0;
                    } else if let Some(ek) = map_editor_key(k) {
                        self.dispatch_theme_editor(ek, k.modifiers.contains(KeyModifiers::SHIFT));
                    }
                }
                Event::Mouse(m) => match m.kind {
                    MouseEventKind::ScrollUp => {
                        self.theme_scroll = self.theme_scroll.saturating_sub(3);
                    }
                    MouseEventKind::ScrollDown => {
                        self.theme_scroll = self
                            .theme_scroll
                            .saturating_add(3)
                            .min(self.theme_scroll_max);
                    }
                    _ => {}
                },
                _ => {}
            }
            return Ok(());
        }

        if let Some(result) = self.handle_modal_event(evt.clone()) {
            match result {
                ModalResult::Continue | ModalResult::CloseSuccess | ModalResult::CloseCancel => {
                    return Ok(());
                }
            }
        }

        match evt {
            Event::Key(k) => match k.code {
                KeyCode::F(1) => self.show_help = true,
                KeyCode::F(2) => {
                    self.show_theme = true;
                    self.theme_editor = Some(ldnddev_theme::ThemeEditor::new(
                        super::theme::palette_from_theme(&self.theme),
                        super::theme::extra_theme_fields(),
                    ));
                }
                KeyCode::F(3) => self.open_validation_modal(),
                KeyCode::Char('p') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.begin_preview();
                }
                KeyCode::Char('E') if k.modifiers.contains(KeyModifiers::SHIFT) => {
                    self.begin_export();
                }
                KeyCode::Char('s') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.begin_save();
                }
                KeyCode::Char('q') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.request_quit();
                }
                KeyCode::Tab | KeyCode::BackTab => {
                    if self.details_visible {
                        self.pane = match self.pane {
                            super::tree::PaneFocus::Structure => super::tree::PaneFocus::Details,
                            super::tree::PaneFocus::Details => super::tree::PaneFocus::Structure,
                        };
                    }
                }
                KeyCode::Enter => self.tree_enter(),
                KeyCode::Char('/') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.open_insert_picker();
                }
                KeyCode::Char('d') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.delete_selected_row();
                }
                KeyCode::Char('y') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.duplicate_selected_row();
                }
                KeyCode::Char('u') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.undo_last();
                }
                KeyCode::Char('J') => self.reorder_selected(1),
                KeyCode::Char('K') => self.reorder_selected(-1),
                KeyCode::Char('C') => self.add_column(),
                KeyCode::Char('V') => self.remove_column(),
                KeyCode::Char('c') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.hop_column(-1);
                }
                KeyCode::Char('v') if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.hop_column(1);
                }
                _ => self.handle_tree_nav(k),
            },
            Event::Mouse(m) => self.handle_tree_mouse(m),
            _ => {}
        }

        Ok(())
    }

    fn handle_tree_nav(&mut self, k: event::KeyEvent) {
        use super::tree::PaneFocus;
        let details = self.pane == PaneFocus::Details && self.details_visible;
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
    }

    fn handle_tree_mouse(&mut self, m: event::MouseEvent) {
        use super::tree::PaneFocus;
        let (x, y) = (m.column, m.row);
        let in_tree = contains(self.tree_area, x, y);
        let in_details = self.details_visible && contains(self.details_area, x, y);
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
                    super::theme::apply_palette(&mut self.theme, &editor.palette);
                }
            }
            ldnddev_theme::EditorOutcome::RequestSave => {
                if let Some(editor) = &self.theme_editor {
                    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    if let Ok(path) = ldnddev_theme::save_theme(
                        &editor.palette,
                        &root,
                        "dd_emailforge_theme.yml",
                        editor.save_target,
                        ldnddev_theme::default_config_home().as_deref(),
                        &editor.fields,
                    ) {
                        super::theme::apply_palette(&mut self.theme, &editor.palette);
                        self.theme_source = editor.save_target.label().to_string();
                        self.theme_editor = None;
                        self.show_theme = false;
                        let _ = path;
                    }
                }
            }
            ldnddev_theme::EditorOutcome::Closed { .. } => {
                if let Some(mut editor) = self.theme_editor.take() {
                    editor.revert();
                    super::theme::apply_palette(&mut self.theme, &editor.palette);
                }
                self.show_theme = false;
            }
            ldnddev_theme::EditorOutcome::HexError(_) | ldnddev_theme::EditorOutcome::None => {}
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
