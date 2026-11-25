//! Headless window tests. Mounting does not draw a frame.

use std::path::PathBuf;

use dd_emailforge::starters::{self, StarterKind};
use gpui::{AppContext, Bounds, SharedString, TestAppContext, px, size};

use crate::settings::{self, Density};
use crate::shell::{Shell, ShellOptions};
use crate::windows::{self, MAIN_APP_ID};

fn opts(dir: &std::path::Path, path: Option<PathBuf>) -> ShellOptions {
    ShellOptions {
        path,
        settings_file: dir.join("gui.json"),
        cwd: dir.to_path_buf(),
        quit_runtime: false,
    }
}

fn mount(cx: &mut TestAppContext, options: ShellOptions) -> gpui::Entity<Shell> {
    cx.update(gpui_component::init);
    cx.new(|cx| Shell::open(options, cx))
}

#[gpui::test]
fn main_window_is_centered_1280_by_800(cx: &mut TestAppContext) {
    cx.update(|cx| {
        let options = windows::main_window_options(Bounds::centered(
            None,
            size(px(1280.0), px(800.0)),
            cx,
        ));
        assert_eq!(options.app_id.as_deref(), Some(MAIN_APP_ID));
        assert_eq!(MAIN_APP_ID, "dev.ldnddev.dd_emailforge");
        let bounds = options.window_bounds.expect("bounds").get_bounds();
        assert_eq!(bounds.size, size(px(1280.0), px(800.0)));
        let title = options
            .titlebar
            .as_ref()
            .and_then(|bar| bar.title.as_ref())
            .map(SharedString::to_string);
        assert_eq!(title.as_deref(), Some("dd_emailforge"));
    });
}

#[gpui::test]
fn settings_sheet_pages_and_second_f1_restores_palette(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let shell = mount(cx, opts(dir.path(), None));
    shell.update(cx, |shell, _| {
        let before = shell.palette_fingerprint();
        let text = shell.visible_text();
        assert!(text.contains("F1: Settings"), "{text}");
        assert!(text.contains("Ctrl+Q: Quit"), "{text}");
        assert!(!text.contains("settings-panel:right:70"), "{text}");
        shell.handle_key("f1");
        let text = shell.visible_text();
        assert!(text.contains("settings-panel:right:70"), "{text}");
        assert!(text.contains("settings-nav:Themes,Window,Help"), "{text}");
        assert!(text.contains("settings-page:Themes"), "{text}");
        assert!(
            text.contains("theme-list:flow") || text.contains("Theme color editor"),
            "{text}"
        );
        assert!(text.contains("> Themes"), "{text}");
        assert!(text.contains("F1: Settings  Esc: Close  Ctrl+Q: Quit"), "{text}");
        if text.contains("Theme color editor") {
            shell.handle_key("right");
        } else {
            shell.handle_key("down");
        }
        shell.handle_key("f1");
        let text = shell.visible_text();
        assert!(
            !text.contains("settings-panel:right:70"),
            "second F1 closes the sheet:\n{text}"
        );
        assert_eq!(shell.palette_fingerprint(), before);
        shell.handle_key("f2");
        let text = shell.visible_text();
        assert!(text.contains("settings-page:Themes"), "{text}");
        shell.handle_key("tab");
        let text = shell.visible_text();
        assert!(text.contains("settings-page:Window"), "{text}");
        assert!(text.contains("Density Comfortable"), "{text}");
        assert!(text.contains("Vim keys on"), "{text}");
        shell.set_density(Density::Compact);
        shell.set_vim_keys(false);
        let text = shell.visible_text();
        assert!(text.contains("Density Compact"), "{text}");
        assert!(text.contains("Vim keys off"), "{text}");
        shell.handle_key("tab");
        let text = shell.visible_text();
        assert!(text.contains("settings-page:Help"), "{text}");
        assert!(text.contains("Settings · F1 · F1"), "{text}");
        assert!(text.contains("Validate · F3 · F3"), "{text}");
        shell.handle_key("escape");
        assert!(!shell.visible_text().contains("settings-panel:right:70"));
        shell.handle_key("q");
        assert!(!shell.quit);
    });
}

#[gpui::test]
fn empty_open_says_no_template_is_open(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let shell = mount(cx, opts(dir.path(), None));
    shell.update(cx, |shell, _| {
        let text = shell.visible_text();
        assert!(text.contains("no template is open"), "{text}");
        assert!(text.contains("n creates a starter"), "{text}");
        assert!(text.contains("o opens a folder"), "{text}");
        assert!(!text.contains("\n> HEAD") && !text.contains("\n  HEAD"), "{text}");
    });
}

#[gpui::test]
fn starter_template_lists_head_brand_or_body(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let json = starters::init_template_dir(dir.path(), StarterKind::Welcome).unwrap();
    let shell = mount(cx, opts(dir.path(), Some(json)));
    shell.update(cx, |shell, _| {
        let text = shell.visible_text();
        assert!(
            text.contains("HEAD") || text.contains("BRAND") || text.contains("BODY"),
            "{text}"
        );
        assert!(
            text.contains("section") || text.contains("column") || text.contains("email-"),
            "{text}"
        );
        shell.handle_key("f3");
        let text = shell.visible_text();
        assert!(text.contains("Validation"), "{text}");
        assert!(!text.contains("settings-panel:right:70"), "{text}");
    });
}

#[gpui::test]
fn gui_json_round_trip_leaves_template_json(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let json = starters::init_template_dir(dir.path(), StarterKind::Newsletter).unwrap();
    let before = std::fs::read(&json).unwrap();
    let shell = mount(cx, opts(dir.path(), Some(json.clone())));
    shell.update(cx, |shell, _| {
        shell.handle_key("f1");
        shell.handle_key("tab");
        shell.set_density(Density::Compact);
        shell.set_vim_keys(false);
        shell.set_reduced_motion(true);
        assert_eq!(
            shell.settings_file_path(),
            dir.path().join("gui.json")
        );
    });
    let loaded = settings::load_from(&dir.path().join("gui.json"));
    assert_eq!(loaded.density, Density::Compact);
    assert!(!loaded.vim_keys);
    assert!(loaded.reduced_motion);
    let text = std::fs::read_to_string(dir.path().join("gui.json")).unwrap();
    assert!(text.contains("compact"));
    assert!(text.contains("vim_keys"));
    assert!(!text.contains("subject"));
    assert_eq!(std::fs::read(&json).unwrap(), before);
    let again = mount(cx, opts(dir.path(), Some(json)));
    again.update(cx, |shell, _| {
        shell.handle_key("f1");
        shell.handle_key("tab");
        let text = shell.visible_text();
        assert!(text.contains("Density Compact"), "{text}");
        assert!(text.contains("Vim keys off"), "{text}");
    });
}
