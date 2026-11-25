//! Editor window: template outline, validation, and the Settings sheet.

use std::path::{Path, PathBuf};

use dd_emailforge::model::Template;
use dd_emailforge::starters::StarterKind;
use gpui::{
    App, ClickEvent, Context, FocusHandle, Focusable, FontWeight, IntoElement, KeyDownEvent,
    ParentElement, Render, ScrollHandle, StatefulInteractiveElement, Styled, Window, div, px,
    prelude::*, relative,
};
use ldnddev_theme::{EditorKey, EditorOutcome, Palette, ThemeEditor, ThemePicker};

use crate::outline;
use crate::settings::{self, Density, GuiSettings};
use crate::shortcuts;
use crate::theme_load::{self, Chrome};

const PAGES: [&str; 3] = ["Themes", "Window", "Help"];
const THEME_FILE: &str = "dd_emailforge_theme.yml";

pub struct ShellOptions {
    pub path: Option<PathBuf>,
    pub settings_file: PathBuf,
    pub cwd: PathBuf,
    pub quit_runtime: bool,
}

impl ShellOptions {
    pub fn production(path: Option<PathBuf>) -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            path,
            settings_file: settings::settings_path(),
            cwd,
            quit_runtime: true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConfirmKind {
    Quit,
    New,
    Open,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pending {
    None,
    New,
    Open,
}

struct Sheet {
    picker: ThemePicker,
    editor: Option<ThemeEditor>,
    direct_editor: bool,
    snapshot: Palette,
    label_snapshot: String,
    list_palette: Palette,
    hex_error: Option<String>,
}

pub struct Shell {
    focus: FocusHandle,
    quit_runtime: bool,
    cwd: PathBuf,
    settings_file: PathBuf,
    settings: GuiSettings,
    palette: Palette,
    theme_label: String,
    theme_warning: Option<String>,
    omarchy_paths: ldnddev_theme::OmarchyPaths,
    template: Option<Template>,
    path: Option<PathBuf>,
    loaded_json: String,
    outline_row: usize,
    validation: Vec<String>,
    validated: bool,
    sheet: Option<Sheet>,
    settings_page: u8,
    settings_row: usize,
    settings_query: String,
    settings_query_on: bool,
    settings_scroll: ScrollHandle,
    confirm: Option<ConfirmKind>,
    pending: Pending,
    toasts: Vec<String>,
    pub quit: bool,
}

impl Shell {
    pub fn open(options: ShellOptions, cx: &mut Context<Self>) -> Self {
        let loaded = theme_load::load_theme();
        let gui = settings::load_from(&options.settings_file);
        let mut shell = Self {
            focus: cx.focus_handle(),
            quit_runtime: options.quit_runtime,
            cwd: options.cwd,
            settings_file: options.settings_file,
            settings: gui,
            palette: loaded.palette,
            theme_label: loaded.label,
            theme_warning: loaded.warning.clone(),
            omarchy_paths: loaded.paths,
            template: None,
            path: None,
            loaded_json: String::new(),
            outline_row: 0,
            validation: Vec::new(),
            validated: false,
            sheet: None,
            settings_page: 0,
            settings_row: 0,
            settings_query: String::new(),
            settings_query_on: false,
            settings_scroll: ScrollHandle::new(),
            confirm: None,
            pending: Pending::None,
            toasts: Vec::new(),
            quit: false,
        };
        if let Some(warning) = loaded.warning {
            shell.toast(warning);
        }
        if let Some(path) = options.path {
            shell.load_path(&path);
        }
        shell
    }

    pub fn handle_key(&mut self, key: &str) {
        let key = key.to_ascii_lowercase();
        if self.confirm.is_some() {
            self.confirm_key(&key);
            return;
        }
        if self.sheet.is_some() && is_settings_toggle(&key) {
            self.revert_sheet();
            return;
        }
        if key == "ctrl-q" {
            self.request_quit();
            return;
        }
        if self.sheet.is_some() {
            self.sheet_key(&key);
            return;
        }
        if is_settings_toggle(&key) {
            self.open_sheet();
            return;
        }
        match key.as_str() {
            "f3" => self.validate_now(),
            "ctrl-s" => self.save_template(),
            "n" => self.request_new(),
            "o" => self.request_open(),
            "up" => self.move_outline(-1),
            "down" => self.move_outline(1),
            "k" if self.settings.vim_keys => self.move_outline(-1),
            "j" if self.settings.vim_keys => self.move_outline(1),
            "q" => {}
            _ => {}
        }
    }

    #[cfg(test)]
    pub fn visible_text(&self) -> String {
        self.lines().join("\n")
    }

    #[cfg(test)]
    pub fn palette_fingerprint(&self) -> String {
        self.palette
            .colors()
            .iter()
            .map(|(key, color)| format!("{key}:{}", color.to_hex()))
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn set_density(&mut self, density: Density) {
        self.settings.density = density;
        let _ = settings::save_to(&self.settings_file, &self.settings);
    }

    pub fn set_vim_keys(&mut self, on: bool) {
        self.settings.vim_keys = on;
        let _ = settings::save_to(&self.settings_file, &self.settings);
    }

    #[cfg(test)]
    pub fn set_reduced_motion(&mut self, on: bool) {
        self.settings.reduced_motion = on;
        let _ = settings::save_to(&self.settings_file, &self.settings);
    }

    #[cfg(test)]
    pub fn settings_file_path(&self) -> PathBuf {
        self.settings_file.clone()
    }

    #[cfg(test)]
    fn lines(&self) -> Vec<String> {
        let mut lines = vec!["dd_emailforge".to_string(), self.footer_text()];
        if let Some(confirm) = self.confirm {
            lines.push(confirm_message(confirm).to_string());
        }
        if self.template.is_none() {
            lines.push("no template is open".to_string());
            lines.push("n creates a starter".to_string());
            lines.push("o opens a folder".to_string());
        } else {
            if let Some(path) = &self.path {
                lines.push(path.display().to_string());
            }
            for (index, row) in self.outline().iter().enumerate() {
                let mark = if index == self.outline_row { ">" } else { " " };
                lines.push(format!("{mark} {row}"));
            }
        }
        if self.validated {
            lines.push("Validation".to_string());
            if self.validation.is_empty() {
                lines.push("Validation: no errors".to_string());
            } else {
                lines.extend(self.validation.clone());
            }
        }
        for toast in &self.toasts {
            lines.push(toast.clone());
        }
        if self.sheet.is_some() {
            lines.push("settings-panel:right:70".to_string());
            lines.push("settings-nav:Themes,Window,Help".to_string());
            let page = if !self.settings_query.trim().is_empty() {
                "Results"
            } else {
                PAGES
                    .get(self.settings_page as usize)
                    .copied()
                    .unwrap_or("Themes")
            };
            lines.push(format!("settings-page:{page}"));
            lines.push("Settings".to_string());
            for (index, label) in PAGES.iter().enumerate() {
                let mark = if self.settings_query.trim().is_empty()
                    && self.settings_page as usize == index
                {
                    ">"
                } else {
                    " "
                };
                lines.push(format!("{mark} {label}"));
            }
            if !self.settings_query.trim().is_empty() {
                lines.push("Results".to_string());
                lines.extend(self.settings_results());
            } else if self.settings_page == 1 {
                lines.push(format!("Density {}", self.settings.density.label()));
                lines.push(format!(
                    "Vim keys {}",
                    if self.settings.vim_keys { "on" } else { "off" }
                ));
            } else if self.settings_page == 2 {
                lines.push("Shortcut sheet".to_string());
                lines.extend(shortcuts::lines());
            } else {
                lines.push(format!("Source: {}", self.theme_label));
                let status = self
                    .theme_warning
                    .clone()
                    .unwrap_or_else(|| "OK (loaded cleanly)".to_string());
                lines.push(format!("Status: {status}"));
                if let Some(sheet) = &self.sheet {
                    lines.push(format!("Save to {}", sheet.picker.save_target.label()));
                    if let Some(editor) = &sheet.editor {
                        lines.push("Theme color editor".to_string());
                        for (index, field) in editor.fields.iter().enumerate() {
                            let hex = editor
                                .palette
                                .get(field.key)
                                .map(|color| color.to_hex())
                                .unwrap_or_else(|| "#000000".to_string());
                            let mark = if index == editor.selected { ">" } else { " " };
                            lines.push(format!("{mark} {} {hex}", field.key));
                        }
                        lines.push(format!("Hex: {}", editor.hex_draft));
                        if let Some(err) = &sheet.hex_error {
                            lines.push(err.clone());
                        }
                    } else {
                        lines.push("theme-list:flow".to_string());
                        for (index, row) in sheet.picker.rows.iter().enumerate() {
                            let mark = if index == sheet.picker.selected {
                                ">"
                            } else {
                                " "
                            };
                            lines.push(format!("{mark} {}", theme_row_label(row)));
                        }
                        lines.push("Edit colors".to_string());
                    }
                    lines.push("Save".to_string());
                }
            }
        }
        lines
    }

    fn outline(&self) -> Vec<String> {
        self.template
            .as_ref()
            .map(outline::outline_lines)
            .unwrap_or_default()
    }

    fn footer_text(&self) -> String {
        if self.sheet.is_some() || self.confirm.is_some() {
            "F1: Settings  Esc: Close  Ctrl+Q: Quit".to_string()
        } else {
            "F1: Settings  Ctrl+Q: Quit".to_string()
        }
    }

    fn row_h(&self) -> f32 {
        self.settings.density.row_px()
    }

    fn toast(&mut self, message: String) {
        self.toasts.push(message);
        if self.toasts.len() > 4 {
            self.toasts.remove(0);
        }
    }

    fn load_path(&mut self, path: &Path) {
        let json = match dd_emailforge::storage::resolve_template_path(path) {
            Ok(json) => json,
            Err(err) => {
                self.toast(format!("{err:#}"));
                self.template = None;
                self.path = None;
                self.loaded_json.clear();
                return;
            }
        };
        match dd_emailforge::storage::load_template(&json) {
            Ok(template) => {
                self.loaded_json = serde_json::to_string(&template).unwrap_or_default();
                self.template = Some(template);
                self.path = Some(json.clone());
                self.outline_row = 0;
                self.validation.clear();
                self.validated = false;
                if self.quit_runtime {
                    dd_emailforge::storage::remember_last_template(&json);
                }
            }
            Err(err) => {
                self.toast(err.to_string());
                self.template = None;
                self.path = None;
                self.loaded_json.clear();
            }
        }
    }

    fn template_dirty(&self) -> bool {
        let Some(template) = &self.template else {
            return false;
        };
        match serde_json::to_string(template) {
            Ok(current) => current != self.loaded_json,
            Err(_) => false,
        }
    }

    fn request_quit(&mut self) {
        if self.template_dirty() {
            self.confirm = Some(ConfirmKind::Quit);
        } else {
            self.quit = true;
        }
    }

    fn request_new(&mut self) {
        if self.template_dirty() {
            self.confirm = Some(ConfirmKind::New);
        } else {
            self.pending = Pending::New;
        }
    }

    fn request_open(&mut self) {
        if self.template_dirty() {
            self.confirm = Some(ConfirmKind::Open);
        } else {
            self.pending = Pending::Open;
        }
    }

    fn confirm_key(&mut self, key: &str) {
        match key {
            "enter" | "y" => {
                let kind = self.confirm.take();
                match kind {
                    Some(ConfirmKind::Quit) => self.quit = true,
                    Some(ConfirmKind::New) => self.pending = Pending::New,
                    Some(ConfirmKind::Open) => self.pending = Pending::Open,
                    None => {}
                }
            }
            "escape" | "n" => self.confirm = None,
            "ctrl-q" if self.confirm == Some(ConfirmKind::Quit) => {
                self.confirm = None;
                self.quit = true;
            }
            _ => {}
        }
    }

    fn save_template(&mut self) {
        let Some(template) = self.template.clone() else {
            self.toast("No template open. n creates a starter and o opens a folder.".into());
            return;
        };
        let Some(path) = self.path.clone() else {
            self.toast("No path to save.".into());
            return;
        };
        match dd_emailforge::storage::save_template(&path, &template) {
            Ok(()) => {
                self.loaded_json = serde_json::to_string(&template).unwrap_or_default();
                self.toast(format!("Saved {}", path.display()));
            }
            Err(err) => self.toast(format!("Failed to save: {err:#}")),
        }
    }

    fn validate_now(&mut self) {
        let Some(template) = &self.template else {
            self.toast("No template open. n creates a starter and o opens a folder.".into());
            return;
        };
        let root = self
            .path
            .as_ref()
            .map(|path| dd_emailforge::storage::template_root(path));
        let report =
            dd_emailforge::validate::validate_template_with_root(template, root.as_deref());
        let mut lines = Vec::new();
        for err in &report.errors {
            lines.push(format!("- {err}"));
        }
        for warn in &report.warnings {
            lines.push(format!("warning: {warn}"));
        }
        let count = report.errors.len();
        self.validation = lines;
        self.validated = true;
        if count == 0 {
            self.toast("Validation: no errors".into());
        } else {
            self.toast(format!("Validation: {count} errors"));
        }
    }

    fn move_outline(&mut self, delta: i32) {
        let len = self.outline().len() as i32;
        if len == 0 {
            return;
        }
        self.outline_row = (self.outline_row as i32 + delta).clamp(0, len - 1) as usize;
    }

    fn open_sheet(&mut self) {
        if self.sheet.is_some() {
            return;
        }
        let snapshot = self.palette.clone();
        let picker = ThemePicker::new(
            snapshot.clone(),
            theme_load::theme_extras(),
            self.omarchy_paths.clone(),
        );
        let direct = !picker.lists_omarchy();
        let editor = if direct {
            Some(editor_from(&picker))
        } else {
            None
        };
        let label_snapshot = self.theme_label.clone();
        self.sheet = Some(Sheet {
            list_palette: picker.palette.clone(),
            picker,
            editor,
            direct_editor: direct,
            snapshot,
            label_snapshot,
            hex_error: None,
        });
        self.settings_page = 0;
        self.settings_row = 0;
        self.settings_query.clear();
        self.settings_query_on = false;
    }

    fn revert_sheet(&mut self) {
        if let Some(sheet) = self.sheet.take() {
            self.palette = sheet.snapshot;
            self.theme_label = sheet.label_snapshot;
        }
        self.settings_query.clear();
        self.settings_query_on = false;
    }

    fn editing_hex(&self) -> bool {
        self.sheet
            .as_ref()
            .and_then(|sheet| sheet.editor.as_ref())
            .is_some_and(|editor| editor.editing_hex)
    }

    fn sheet_key(&mut self, key: &str) {
        if self.settings_query_on {
            self.query_key(key);
            return;
        }
        if key == "/" && !self.editing_hex() {
            self.settings_query_on = true;
            self.settings_query.clear();
            self.settings_row = 0;
            return;
        }
        if let Some(dir) = page_dir(key, self.editing_hex()) {
            self.jump_page(dir);
            return;
        }
        if self.settings_page == 0 && key == "t" && !self.editing_hex() {
            self.toggle_save_target();
            return;
        }
        if self.settings_page == 0 && self.sheet.as_ref().is_some_and(|sheet| sheet.editor.is_some())
        {
            self.editor_key(key);
            return;
        }
        let vim = self.settings.vim_keys;
        if self.settings_page == 0 {
            match key {
                "escape" => self.sheet_escape(),
                "enter" => self.apply_theme(),
                "e" => self.open_color_editor(),
                "s" | "y" => self.save_theme(),
                "up" => self.move_theme(-1),
                "down" => self.move_theme(1),
                "pageup" => self.move_theme(-8),
                "pagedown" => self.move_theme(8),
                "k" if vim => self.move_theme(-1),
                "j" if vim => self.move_theme(1),
                "g" if vim => self.move_theme_edge(true),
                "shift-g" if vim => self.move_theme_edge(false),
                _ => {}
            }
            return;
        }
        if self.settings_page == 1 {
            let last = 1usize;
            match key {
                "escape" => self.sheet_escape(),
                "up" => self.settings_row = self.settings_row.saturating_sub(1),
                "down" => self.settings_row = (self.settings_row + 1).min(last),
                "k" if vim => self.settings_row = self.settings_row.saturating_sub(1),
                "j" if vim => self.settings_row = (self.settings_row + 1).min(last),
                "enter" => self.activate_window_row(),
                _ => {}
            }
            return;
        }
        if key == "escape" {
            self.sheet_escape();
        }
    }

    fn sheet_escape(&mut self) {
        if self.sheet.as_ref().is_some_and(|sheet| sheet.editor.is_some()) {
            let was_hex = self.editing_hex();
            let outcome = self.editor_handle(EditorKey::Esc);
            if was_hex && matches!(outcome, Some(EditorOutcome::None)) {
                return;
            }
            if matches!(outcome, Some(EditorOutcome::Closed { .. })) {
                let direct = self
                    .sheet
                    .as_ref()
                    .is_some_and(|sheet| sheet.direct_editor);
                let list = self
                    .sheet
                    .as_ref()
                    .map(|sheet| sheet.list_palette.clone());
                if let Some(sheet) = &mut self.sheet {
                    sheet.editor = None;
                    sheet.hex_error = None;
                }
                if direct {
                    self.revert_sheet();
                } else if let Some(list) = list {
                    self.palette = list;
                }
                return;
            }
        }
        if self.settings_query_on || !self.settings_query.trim().is_empty() {
            self.settings_query.clear();
            self.settings_query_on = false;
            return;
        }
        self.revert_sheet();
    }

    fn jump_page(&mut self, dir: i32) {
        let next = (self.settings_page as i32 + dir).clamp(0, (PAGES.len() - 1) as i32) as u8;
        if next != self.settings_page {
            self.settings_page = next;
            self.settings_row = 0;
        }
    }

    fn query_key(&mut self, key: &str) {
        match key {
            "escape" => {
                self.settings_query_on = false;
                self.settings_query.clear();
                self.settings_row = 0;
            }
            "backspace" => {
                self.settings_query.pop();
                self.settings_row = 0;
            }
            "enter" => self.activate_result(),
            "up" => self.settings_row = self.settings_row.saturating_sub(1),
            "down" => {
                let count = self.settings_results().len();
                if count > 0 {
                    self.settings_row = (self.settings_row + 1).min(count - 1);
                }
            }
            other if other.chars().count() == 1 && !other.chars().any(|ch| ch.is_control()) => {
                self.settings_query.push_str(other);
                self.settings_row = 0;
            }
            _ => {}
        }
    }

    fn settings_results(&self) -> Vec<String> {
        let query = self.settings_query.trim().to_ascii_lowercase();
        if query.is_empty() {
            return Vec::new();
        }
        let mut hits = Vec::new();
        if let Some(sheet) = &self.sheet {
            for row in &sheet.picker.rows {
                let label = theme_row_label(row);
                if label.to_ascii_lowercase().contains(&query) {
                    hits.push(label);
                }
            }
        }
        for label in ["Density", "Vim keys"] {
            if label.to_ascii_lowercase().contains(&query) {
                hits.push(label.to_string());
            }
        }
        for row in shortcuts::rows() {
            if row.action.to_ascii_lowercase().contains(&query) {
                hits.push(row.action.to_string());
            }
        }
        hits
    }

    fn activate_result(&mut self) {
        let Some(label) = self.settings_results().get(self.settings_row).cloned() else {
            return;
        };
        self.settings_query_on = false;
        self.settings_query.clear();
        if label == "Density" || label == "Vim keys" {
            self.settings_page = 1;
            self.settings_row = usize::from(label == "Vim keys");
            self.activate_window_row();
            return;
        }
        if let Some(sheet) = &self.sheet
            && let Some(index) = sheet
                .picker
                .rows
                .iter()
                .position(|row| theme_row_label(row) == label)
        {
            self.settings_page = 0;
            self.choose_theme(index);
            return;
        }
        self.settings_page = 2;
        self.settings_row = 0;
    }

    fn activate_window_row(&mut self) {
        if self.settings_row == 0 {
            self.set_density(self.settings.density.toggled());
        } else {
            self.set_vim_keys(!self.settings.vim_keys);
        }
    }

    fn choose_theme(&mut self, index: usize) {
        if self
            .sheet
            .as_ref()
            .is_some_and(|sheet| sheet.editor.is_some())
        {
            return;
        }
        if let Some(sheet) = &mut self.sheet {
            sheet.picker.select(index);
            let _ = sheet.picker.preview_selected();
        }
        self.install_picker_palette();
        self.settings_scroll.scroll_to_item(index);
    }

    fn move_theme(&mut self, delta: i32) {
        let next = {
            let Some(sheet) = &mut self.sheet else {
                return;
            };
            if sheet.editor.is_some() {
                return;
            }
            let len = sheet.picker.rows.len() as i32;
            if len == 0 {
                return;
            }
            let next = (sheet.picker.selected as i32 + delta).clamp(0, len - 1) as usize;
            sheet.picker.select(next);
            let _ = sheet.picker.preview_selected();
            next
        };
        self.install_picker_palette();
        self.settings_scroll.scroll_to_item(next);
    }

    fn move_theme_edge(&mut self, first: bool) {
        let next = {
            let Some(sheet) = &mut self.sheet else {
                return;
            };
            let last = sheet.picker.rows.len().saturating_sub(1);
            let next = if first { 0 } else { last };
            sheet.picker.select(next);
            let _ = sheet.picker.preview_selected();
            next
        };
        self.install_picker_palette();
        self.settings_scroll.scroll_to_item(next);
    }

    fn apply_theme(&mut self) {
        if let Some(sheet) = &mut self.sheet {
            if sheet.editor.is_some() {
                return;
            }
            let _ = sheet.picker.preview_selected();
        }
        self.install_picker_palette();
    }

    fn install_picker_palette(&mut self) {
        if let Some(sheet) = &self.sheet {
            self.palette = sheet.picker.palette.clone();
            self.theme_label = sheet
                .picker
                .rows
                .get(sheet.picker.selected)
                .map(theme_row_label)
                .unwrap_or_else(|| self.theme_label.clone());
        }
    }

    fn toggle_save_target(&mut self) {
        let Some(sheet) = &mut self.sheet else {
            return;
        };
        sheet.picker.save_target = sheet.picker.save_target.toggled();
        let target = sheet.picker.save_target;
        if let Some(editor) = &mut sheet.editor {
            editor.save_target = target;
        }
    }

    fn open_color_editor(&mut self) {
        let Some(sheet) = &mut self.sheet else {
            return;
        };
        if sheet.editor.is_some() {
            return;
        }
        sheet.list_palette = sheet.picker.palette.clone();
        sheet.editor = Some(editor_from(&sheet.picker));
        sheet.hex_error = None;
    }

    fn editor_key(&mut self, key: &str) {
        let shift = key.starts_with("shift-");
        let Some(editor_key) = editor_key_from(key, self.settings.vim_keys) else {
            return;
        };
        let outcome = self.editor_handle_shift(editor_key, shift);
        match outcome {
            Some(EditorOutcome::RequestSave) => self.save_theme(),
            Some(EditorOutcome::Closed { .. }) => {
                let direct = self
                    .sheet
                    .as_ref()
                    .is_some_and(|sheet| sheet.direct_editor);
                let list = self
                    .sheet
                    .as_ref()
                    .map(|sheet| sheet.list_palette.clone());
                if let Some(sheet) = &mut self.sheet {
                    sheet.editor = None;
                    sheet.hex_error = None;
                }
                if direct {
                    self.revert_sheet();
                } else if let Some(list) = list {
                    self.palette = list;
                }
            }
            Some(EditorOutcome::HexError(message)) => {
                if let Some(sheet) = &mut self.sheet {
                    sheet.hex_error = Some(message);
                }
            }
            Some(EditorOutcome::PaletteChanged) => {
                if let Some(sheet) = &mut self.sheet {
                    if let Some(editor) = &sheet.editor {
                        let palette = editor.palette.clone();
                        sheet.picker.palette = palette.clone();
                        self.palette = palette;
                    }
                    sheet.hex_error = None;
                }
            }
            Some(EditorOutcome::None) | None => {
                if let Some(sheet) = &mut self.sheet
                    && sheet
                        .editor
                        .as_ref()
                        .is_some_and(|editor| editor.editing_hex)
                {
                    sheet.hex_error = None;
                }
            }
        }
    }

    fn editor_handle(&mut self, key: EditorKey) -> Option<EditorOutcome> {
        self.editor_handle_shift(key, false)
    }

    fn editor_handle_shift(&mut self, key: EditorKey, shift: bool) -> Option<EditorOutcome> {
        self.sheet
            .as_mut()
            .and_then(|sheet| sheet.editor.as_mut())
            .map(|editor| editor.handle(key, shift))
    }

    fn save_theme(&mut self) {
        let Some(sheet) = &self.sheet else {
            return;
        };
        let palette = sheet
            .editor
            .as_ref()
            .map(|editor| editor.palette.clone())
            .unwrap_or_else(|| sheet.picker.palette.clone());
        let target = sheet
            .editor
            .as_ref()
            .map(|editor| editor.save_target)
            .unwrap_or(sheet.picker.save_target);
        match ldnddev_theme::save_theme(
            &palette,
            &self.cwd,
            THEME_FILE,
            target,
            ldnddev_theme::default_config_home().as_deref(),
            &theme_load::theme_fields(),
        ) {
            Ok(path) => {
                self.theme_label = target.label().to_string();
                self.toast(format!("Theme saved ({})", path.display()));
            }
            Err(err) => self.toast(format!("Theme save failed: {err}")),
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        self.handle_key(&chord(event));
        self.flush_pending(cx);
        if self.quit && self.quit_runtime {
            cx.quit();
        }
    }

    fn flush_pending(&mut self, cx: &mut Context<Self>) {
        let pending = self.pending;
        self.pending = Pending::None;
        match pending {
            Pending::New => self.prompt_new(cx),
            Pending::Open => self.prompt_open(cx),
            Pending::None => {}
        }
    }

    fn prompt_new(&mut self, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("New template folder".into()),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = receiver.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            this.update(cx, |shell, cx| {
                match dd_emailforge::starters::init_template_dir(&path, StarterKind::Welcome) {
                    Ok(json) => shell.load_path(&json),
                    Err(err) => shell.toast(format!("{err:#}")),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn prompt_open(&mut self, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: true,
            directories: true,
            multiple: false,
            prompt: Some("Open template folder".into()),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = receiver.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            this.update(cx, |shell, cx| {
                shell.load_path(&path);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

}

impl Focusable for Shell {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.focus.is_focused(window) {
            self.focus.focus(window, cx);
        }
        let colors = theme_load::chrome(&self.palette);
        let row_h = self.row_h();
        let footer = self.footer_text();
        let outline = self.outline();
        let outline_row = self.outline_row;
        let empty = self.template.is_none();
        let path = self
            .path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_default();
        let validation = self.validation.clone();
        let validated = self.validated;
        let toasts = self.toasts.clone();
        let confirm = self.confirm;
        let sheet = self.sheet_view();
        let page = self.settings_page;
        let query = self.settings_query.clone();
        let scroll = self.settings_scroll.clone();

        div()
            .id("shell")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|shell, event: &KeyDownEvent, _, cx| {
                shell.on_key_down(event, cx);
                cx.notify();
            }))
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .bg(colors.base)
            .text_color(colors.text)
            .text_size(px(13.0))
            .child(band(
                "dd_emailforge",
                row_h,
                colors.raised,
                colors.text,
                colors.border,
            ))
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.0))
                    .relative()
                    .overflow_hidden()
                    .child(outline_body(
                        empty,
                        &path,
                        &outline,
                        outline_row,
                        validated,
                        &validation,
                        colors,
                    )),
            )
            .child(band(
                &footer,
                row_h,
                colors.raised,
                colors.secondary,
                colors.border,
            ))
            .when(!toasts.is_empty(), |shell| {
                shell.child(toast_stack(&toasts, colors))
            })
            .when(confirm.is_some(), |shell| {
                shell.child(confirm_box(
                    confirm_message(confirm.unwrap_or(ConfirmKind::Quit)),
                    colors,
                    cx,
                ))
            })
            .when(sheet.is_some(), |shell| {
                shell.child(settings_panel(
                    cx,
                    sheet.as_ref().unwrap(),
                    page,
                    &query,
                    row_h,
                    &scroll,
                    colors,
                ))
            })
    }
}

struct SheetView {
    page_name: String,
    nav_on: [bool; 3],
    source: String,
    status: String,
    save_label: String,
    themes: Vec<(usize, String, bool)>,
    colors: Option<ColorView>,
    direct: bool,
    density: String,
    vim: String,
    window_row: usize,
    query: String,
    results: Vec<String>,
    result_row: usize,
}

struct ColorView {
    rows: Vec<(usize, String, bool)>,
    hex: String,
    error: Option<String>,
}

impl Shell {
    fn sheet_view(&self) -> Option<SheetView> {
        let sheet = self.sheet.as_ref()?;
        let page_name = if !self.settings_query.trim().is_empty() {
            "Results".to_string()
        } else {
            PAGES
                .get(self.settings_page as usize)
                .copied()
                .unwrap_or("Themes")
                .to_string()
        };
        let mut nav_on = [false; 3];
        if self.settings_query.trim().is_empty()
            && let Some(flag) = nav_on.get_mut(self.settings_page as usize)
        {
            *flag = true;
        }
        let colors = sheet.editor.as_ref().map(|editor| ColorView {
            rows: editor
                .fields
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let hex = editor
                        .palette
                        .get(field.key)
                        .map(|color| color.to_hex())
                        .unwrap_or_else(|| "#000000".to_string());
                    (
                        index,
                        format!("{} {hex}", field.key),
                        index == editor.selected,
                    )
                })
                .collect(),
            hex: editor.hex_draft.clone(),
            error: sheet.hex_error.clone(),
        });
        Some(SheetView {
            page_name,
            nav_on,
            source: self.theme_label.clone(),
            status: self
                .theme_warning
                .clone()
                .unwrap_or_else(|| "OK (loaded cleanly)".to_string()),
            save_label: sheet.picker.save_target.label().to_string(),
            themes: sheet
                .picker
                .rows
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    (
                        index,
                        theme_row_label(row),
                        index == sheet.picker.selected,
                    )
                })
                .collect(),
            colors,
            direct: sheet.direct_editor,
            density: self.settings.density.label().to_string(),
            vim: if self.settings.vim_keys { "on" } else { "off" }.to_string(),
            window_row: self.settings_row,
            query: self.settings_query.clone(),
            results: self.settings_results(),
            result_row: self.settings_row,
        })
    }
}

fn band(text: &str, height: f32, bg: gpui::Hsla, fg: gpui::Hsla, border: gpui::Hsla) -> gpui::Div {
    div()
        .h(px(height))
        .w_full()
        .flex_shrink_0()
        .px(px(12.0))
        .flex()
        .items_center()
        .bg(bg)
        .text_color(fg)
        .border_b_1()
        .border_color(border)
        .font_weight(FontWeight::BOLD)
        .child(text.to_string())
}

fn outline_body(
    empty: bool,
    path: &str,
    outline: &[String],
    selected: usize,
    validated: bool,
    validation: &[String],
    colors: Chrome,
) -> gpui::Stateful<gpui::Div> {
    let mut column = div().flex().flex_col().p(px(8.0)).gap(px(2.0));
    if empty {
        column = column
            .child(line("no template is open", colors.text))
            .child(line(
                "n creates a starter and o opens a folder",
                colors.secondary,
            ));
    } else {
        if !path.is_empty() {
            column = column.child(line(path, colors.secondary));
        }
        for (index, row) in outline.iter().enumerate() {
            let on = index == selected;
            column = column.child(
                div()
                    .h(px(28.0))
                    .px(px(8.0))
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .when(on, |row| row.bg(colors.selected).text_color(colors.text))
                    .child(row.clone()),
            );
        }
    }
    if validated {
        column = column.child(line("Validation", colors.text));
        if validation.is_empty() {
            column = column.child(line("Validation: no errors", colors.secondary));
        } else {
            for row in validation {
                column = column.child(line(row, colors.secondary));
            }
        }
    }
    div()
        .id("outline-scroll")
        .absolute()
        .size_full()
        .overflow_y_scroll()
        .child(column)
}

fn line(text: &str, color: gpui::Hsla) -> gpui::Div {
    div().text_color(color).child(text.to_string())
}

fn toast_stack(toasts: &[String], colors: Chrome) -> gpui::Div {
    div()
        .absolute()
        .bottom(px(36.0))
        .left(px(12.0))
        .flex()
        .flex_col()
        .gap(px(4.0))
        .children(toasts.iter().map(|toast| {
            div()
                .px(px(8.0))
                .py(px(4.0))
                .bg(colors.panel)
                .border_1()
                .border_color(colors.accent)
                .text_color(colors.text)
                .child(toast.clone())
        }))
}

fn confirm_box(
    message: &str,
    colors: Chrome,
    cx: &mut Context<Shell>,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id("confirm-scrim")
        .absolute()
        .top(px(28.0))
        .bottom(px(28.0))
        .left(px(0.0))
        .right(px(0.0))
        .flex()
        .items_center()
        .justify_center()
        .occlude()
        .on_click(cx.listener(|_shell, _: &ClickEvent, _, cx| {
            cx.stop_propagation();
        }))
        .child(
            div()
                .id("quit-confirm")
                .w(px(420.0))
                .p(px(16.0))
                .bg(colors.panel)
                .border_1()
                .border_color(colors.border)
                .text_color(colors.text)
                .child(message.to_string())
                .child(div().h(px(8.0)))
                .child("Enter confirms. Esc cancels.".to_string()),
        )
}

fn settings_panel(
    cx: &mut Context<Shell>,
    sheet: &SheetView,
    page: u8,
    query: &str,
    row_h: f32,
    scroll: &ScrollHandle,
    colors: Chrome,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id("settings-panel")
        .occlude()
        .absolute()
        .top(px(row_h))
        .right(px(0.0))
        .bottom(px(row_h))
        .w(relative(0.7))
        .flex()
        .flex_col()
        .overflow_hidden()
        .bg(colors.panel)
        .text_color(colors.text)
        .border_l_1()
        .border_color(colors.border)
        .on_click(cx.listener(|_shell, _: &ClickEvent, _, cx| {
            cx.stop_propagation();
        }))
        .child(settings_header(cx, colors))
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .flex()
                .flex_row()
                .overflow_hidden()
                .child(settings_nav(cx, sheet, colors))
                .child(settings_page(cx, sheet, page, query, scroll, colors)),
        )
}

fn settings_header(cx: &mut Context<Shell>, colors: Chrome) -> gpui::Div {
    div()
        .h(px(48.0))
        .px(px(16.0))
        .flex()
        .items_center()
        .gap(px(12.0))
        .border_b_1()
        .border_color(colors.border)
        .child("Settings")
        .child(div().flex_1())
        .child(div().text_color(colors.secondary).child("F1"))
        .child(
            div()
                .id("settings-close")
                .px(px(8.0))
                .cursor_pointer()
                .child("Close")
                .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                    shell.revert_sheet();
                    cx.stop_propagation();
                    cx.notify();
                })),
        )
}

fn settings_nav(cx: &mut Context<Shell>, sheet: &SheetView, colors: Chrome) -> gpui::Div {
    let query = if sheet.query.is_empty() {
        "Search".to_string()
    } else {
        sheet.query.clone()
    };
    let mut column = div()
        .w(px(200.0))
        .h_full()
        .flex()
        .flex_col()
        .overflow_hidden()
        .border_r_1()
        .border_color(colors.border)
        .child(
            div()
                .id("settings-search")
                .h(px(36.0))
                .px(px(8.0))
                .flex()
                .items_center()
                .cursor_pointer()
                .text_color(colors.secondary)
                .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                    shell.settings_query_on = true;
                    cx.stop_propagation();
                    cx.notify();
                }))
                .child(query),
        );
    for (index, label) in PAGES.iter().enumerate() {
        let on = sheet.nav_on[index];
        let mark = if on { ">" } else { " " };
        let nav = index as u8;
        column = column.child(
            div()
                .id(("settings-nav", index))
                .h(px(32.0))
                .px(px(8.0))
                .flex()
                .items_center()
                .overflow_hidden()
                .whitespace_nowrap()
                .cursor_pointer()
                .when(on, |row| row.bg(colors.selected))
                .on_click(cx.listener(move |shell, _: &ClickEvent, _, cx| {
                    shell.settings_page = nav;
                    shell.settings_row = 0;
                    shell.settings_query.clear();
                    shell.settings_query_on = false;
                    cx.stop_propagation();
                    cx.notify();
                }))
                .child(format!("{mark} {label}")),
        );
    }
    column
}

fn settings_page(
    cx: &mut Context<Shell>,
    sheet: &SheetView,
    page: u8,
    query: &str,
    scroll: &ScrollHandle,
    colors: Chrome,
) -> gpui::Div {
    let hint = if !query.trim().is_empty() {
        "type to filter    up/down move    Enter    Esc clear"
    } else if page == 0 && sheet.colors.is_some() {
        "j/k move    y save    t target    Esc back"
    } else if page == 2 {
        "Esc close"
    } else if page == 0 {
        "j/k move    Enter    t target    e colors    y save    Esc revert"
    } else {
        "j/k move    Enter    Esc close"
    };
    div()
        .flex_1()
        .min_w(px(0.0))
        .h_full()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(
            div()
                .h(px(28.0))
                .px(px(8.0))
                .flex()
                .items_center()
                .font_weight(FontWeight::BOLD)
                .bg(colors.raised)
                .border_b_1()
                .border_color(colors.border)
                .child(sheet.page_name.clone()),
        )
        .child(
            div()
                .px(px(8.0))
                .py(px(4.0))
                .text_color(colors.secondary)
                .child(hint.to_string()),
        )
        .child(settings_scroll(cx, sheet, page, query, scroll, colors))
}

fn settings_scroll(
    cx: &mut Context<Shell>,
    sheet: &SheetView,
    page: u8,
    query: &str,
    scroll: &ScrollHandle,
    colors: Chrome,
) -> gpui::Stateful<gpui::Div> {
    let mut body = div()
        .id("settings-scroll")
        .track_scroll(scroll)
        .flex_1()
        .min_h(px(0.0))
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .p(px(8.0));
    if !query.trim().is_empty() {
        for (index, label) in sheet.results.iter().enumerate() {
            let on = index == sheet.result_row;
            body = body.child(text_row(
                ("settings-result", index),
                label,
                on,
                colors,
            ));
        }
        return body;
    }
    if page == 1 {
        return body
            .child(outlined(
                "Density",
                colors,
                div().flex().flex_col().child(choice_row(
                    cx,
                    "density-comfortable",
                    "Comfortable",
                    sheet.density == "Comfortable",
                    sheet.window_row == 0,
                    colors,
                    |shell| shell.set_density(Density::Comfortable),
                ))
                .child(choice_row(
                    cx,
                    "density-compact",
                    "Compact",
                    sheet.density == "Compact",
                    false,
                    colors,
                    |shell| shell.set_density(Density::Compact),
                )),
            ))
            .child(div().h(px(12.0)))
            .child(outlined(
                "Vim",
                colors,
                div().flex().flex_col().child(choice_row(
                    cx,
                    "vim-keys",
                    &format!("Vim keys {}", sheet.vim),
                    sheet.window_row == 1,
                    sheet.window_row == 1,
                    colors,
                    |shell| shell.set_vim_keys(!shell.settings.vim_keys),
                )),
            ));
    }
    if page == 2 {
        return body.child(outlined(
            "Keyboard shortcuts",
            colors,
            shortcut_column(colors),
        ));
    }
    if let Some(colors_view) = &sheet.colors {
        return body.child(outlined(
            "Colors",
            colors,
            div()
                .flex()
                .flex_col()
                .children(colors_view.rows.iter().map(|(index, label, on)| {
                    let index = *index;
                    div()
                        .id(("color-row", index))
                        .h(px(28.0))
                        .px(px(8.0))
                        .flex()
                        .items_center()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .cursor_pointer()
                        .when(*on, |row| row.bg(colors.selected))
                        .on_click(cx.listener(move |shell, _: &ClickEvent, _, cx| {
                            if let Some(editor) = shell
                                .sheet
                                .as_mut()
                                .and_then(|sheet| sheet.editor.as_mut())
                            {
                                editor.select(index);
                            }
                            cx.stop_propagation();
                            cx.notify();
                        }))
                        .child(label.clone())
                }))
                .child(line(&format!("Hex: {}", colors_view.hex), colors.text))
                .when(colors_view.error.is_some(), |col| {
                    col.child(line(
                        colors_view.error.as_deref().unwrap_or(""),
                        colors.accent,
                    ))
                })
                .child(line(
                    if sheet.direct { "Revert" } else { "Back" },
                    colors.secondary,
                )),
        ));
    }
    for (index, label, on) in &sheet.themes {
        let index = *index;
        body = body.child(
            div()
                .id(("theme-row", index))
                .h(px(28.0))
                .px(px(8.0))
                .flex()
                .items_center()
                .overflow_hidden()
                .whitespace_nowrap()
                .cursor_pointer()
                .when(*on, |row| row.bg(colors.selected))
                .on_click(cx.listener(move |shell, _: &ClickEvent, _, cx| {
                    shell.choose_theme(index);
                    cx.stop_propagation();
                    cx.notify();
                }))
                .child(label.clone()),
        );
    }
    body.child(line(
        &format!("Source: {} · Status: {}", sheet.source, sheet.status),
        colors.secondary,
    ))
    .child(line(
        &format!("Save to {}", sheet.save_label),
        colors.text,
    ))
    .child(
        div()
            .id("edit-colors")
            .h(px(28.0))
            .px(px(8.0))
            .flex()
            .items_center()
            .cursor_pointer()
            .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                shell.open_color_editor();
                cx.stop_propagation();
                cx.notify();
            }))
            .child("Edit colors"),
    )
    .child(
        div()
            .id("save-theme")
            .h(px(28.0))
            .px(px(8.0))
            .flex()
            .items_center()
            .cursor_pointer()
            .on_click(cx.listener(|shell, _: &ClickEvent, _, cx| {
                shell.save_theme();
                cx.stop_propagation();
                cx.notify();
            }))
            .child("Save"),
    )
}

fn shortcut_column(colors: Chrome) -> gpui::Div {
    let mut column = div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(line(
            "GUI key, then TUI key.",
            colors.secondary,
        ));
    for row in shortcuts::lines() {
        column = column.child(
            div()
                .h(px(24.0))
                .flex()
                .items_center()
                .overflow_hidden()
                .whitespace_nowrap()
                .child(row),
        );
    }
    column
}

fn outlined(title: &str, colors: Chrome, body: gpui::Div) -> gpui::Div {
    div()
        .w_full()
        .border_1()
        .border_color(colors.border)
        .flex()
        .flex_col()
        .child(
            div()
                .h(px(28.0))
                .px(px(8.0))
                .flex()
                .items_center()
                .overflow_hidden()
                .whitespace_nowrap()
                .bg(colors.raised)
                .text_color(colors.text)
                .font_weight(FontWeight::BOLD)
                .border_b_1()
                .border_color(colors.border)
                .child(title.to_string()),
        )
        .child(div().p(px(8.0)).child(body))
}

fn choice_row(
    cx: &mut Context<Shell>,
    id: &'static str,
    label: &str,
    on: bool,
    selected: bool,
    colors: Chrome,
    apply: fn(&mut Shell),
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .h(px(28.0))
        .px(px(8.0))
        .flex()
        .items_center()
        .cursor_pointer()
        .when(on || selected, |row| row.bg(colors.selected))
        .on_click(cx.listener(move |shell, _: &ClickEvent, _, cx| {
            apply(shell);
            cx.stop_propagation();
            cx.notify();
        }))
        .child(label.to_string())
}

fn text_row(
    id: (&'static str, usize),
    label: &str,
    on: bool,
    colors: Chrome,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .h(px(28.0))
        .px(px(8.0))
        .flex()
        .items_center()
        .overflow_hidden()
        .whitespace_nowrap()
        .when(on, |row| row.bg(colors.selected))
        .child(label.to_string())
}

fn confirm_message(kind: ConfirmKind) -> &'static str {
    match kind {
        ConfirmKind::Quit => "Unsaved changes. Quit anyway?",
        ConfirmKind::New => "Unsaved changes. New template anyway?",
        ConfirmKind::Open => "Unsaved changes. Open another template anyway?",
    }
}

fn is_settings_toggle(key: &str) -> bool {
    matches!(key, "f1" | "f2" | "ctrl-/" | "ctrl-,")
}

fn page_dir(key: &str, editing_hex: bool) -> Option<i32> {
    match key {
        "tab" => Some(1),
        "shift-tab" | "backtab" => Some(-1),
        "]" if !editing_hex => Some(1),
        "[" if !editing_hex => Some(-1),
        _ => None,
    }
}

fn editor_from(picker: &ThemePicker) -> ThemeEditor {
    let mut editor = ThemeEditor::new(picker.palette.clone(), theme_load::theme_extras());
    editor.save_target = picker.save_target;
    editor
}

fn editor_key_from(key: &str, vim: bool) -> Option<EditorKey> {
    let shift = key.starts_with("shift-");
    let bare = key.strip_prefix("shift-").unwrap_or(key);
    match bare {
        "escape" => Some(EditorKey::Esc),
        "enter" => Some(EditorKey::Enter),
        "backspace" => Some(EditorKey::Backspace),
        "tab" => Some(EditorKey::Tab),
        "up" => Some(EditorKey::Up),
        "down" => Some(EditorKey::Down),
        "left" => Some(EditorKey::Left),
        "right" => Some(EditorKey::Right),
        "h" => Some(EditorKey::Char('h')),
        "l" => Some(EditorKey::Char('l')),
        "j" if vim => Some(EditorKey::Char('j')),
        "k" if vim => Some(EditorKey::Char('k')),
        "s" | "y" => Some(EditorKey::Char('s')),
        "r" if shift => Some(EditorKey::Char('R')),
        "g" if shift => Some(EditorKey::Char('G')),
        "-" | "_" => Some(EditorKey::Char('-')),
        "=" | "+" => Some(EditorKey::Char('+')),
        "[" => Some(EditorKey::Char('[')),
        "]" => Some(EditorKey::Char(']')),
        other if other.chars().count() == 1 && !other.chars().any(|ch| ch.is_control()) => {
            other.chars().next().map(EditorKey::Char)
        }
        _ => None,
    }
}

fn theme_row_label(row: &ldnddev_theme::PickerRow) -> String {
    match row {
        ldnddev_theme::PickerRow::Follow { name } => format!("Follow desktop {name}"),
        ldnddev_theme::PickerRow::Theme(theme) => {
            format!("{} {}", theme.name, theme.mode.label())
        }
        ldnddev_theme::PickerRow::Custom => "Custom built-in".to_string(),
    }
}

fn chord(event: &KeyDownEvent) -> String {
    let mut parts = Vec::new();
    let mods = event.keystroke.modifiers;
    if mods.control || mods.platform {
        parts.push("ctrl".to_string());
    }
    if mods.shift {
        parts.push("shift".to_string());
    }
    if mods.alt {
        parts.push("alt".to_string());
    }
    parts.push(event.keystroke.key.to_ascii_lowercase());
    parts.join("-")
}
