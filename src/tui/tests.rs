use super::*;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn send_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    app.handle_event(Event::Key(KeyEvent::new(code, modifiers)))
        .expect("key event should be handled");
}

fn send_mouse(app: &mut App, kind: MouseEventKind, column: u16, row: u16) {
    app.handle_event(Event::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }))
    .expect("mouse event should be handled");
}

fn chrome_app() -> App {
    App::new(AppTheme::default(), "default".to_string(), None, None, None)
}

fn app_with_template() -> App {
    App::new(
        AppTheme::default(),
        "default".to_string(),
        None,
        Some(crate::model::Template::minimal()),
        None,
    )
}

#[test]
fn f1_opens_and_closes() {
    let mut app = chrome_app();
    send_key(&mut app, KeyCode::F(1), KeyModifiers::NONE);
    assert!(app.show_help);
    send_key(&mut app, KeyCode::F(1), KeyModifiers::NONE);
    assert!(!app.show_help);
}

#[test]
fn f2_opens_and_closes_with_f2_and_esc() {
    let mut app = chrome_app();
    send_key(&mut app, KeyCode::F(2), KeyModifiers::NONE);
    assert!(app.show_theme);
    assert_eq!(app.theme_scroll, 0);
    send_key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!app.show_theme);
}

#[test]
fn f2_scroll_keys_update_theme_scroll() {
    let mut app = chrome_app();
    send_key(&mut app, KeyCode::F(2), KeyModifiers::NONE);
    app.theme_scroll_max = 5;
    send_key(&mut app, KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(app.theme_scroll, 1);
    send_key(&mut app, KeyCode::Char('g'), KeyModifiers::NONE);
    assert_eq!(app.theme_scroll, 0);
}

#[test]
fn ctrl_q_quits() {
    let mut app = chrome_app();
    send_key(&mut app, KeyCode::Char('q'), KeyModifiers::CONTROL);
    assert!(app.should_quit);
}

#[test]
fn q_without_modifiers_does_not_quit() {
    let mut app = chrome_app();
    send_key(&mut app, KeyCode::Char('q'), KeyModifiers::NONE);
    assert!(!app.should_quit);
}

#[test]
fn footer_hint_starts_with_f1_help_and_ctrl_q() {
    let app = chrome_app();
    for width in [40u16, 80, 160] {
        let hint = app.footer_hint(width);
        assert!(
            hint.starts_with("F1:Help") || hint.starts_with("F1: Help"),
            "width {width}: {hint}"
        );
        assert!(
            hint.contains("C-q:Quit") || hint.contains("Ctrl+Q"),
            "width {width}: {hint}"
        );
        assert!(
            !hint.contains("q:Quit")
                || hint.contains("C-q:Quit")
                || hint.contains("Ctrl+Q:Quit")
                || hint.contains("Ctrl+Q: Quit"),
            "bare q:Quit must not appear, width {width}: {hint}"
        );
        let stripped = hint
            .replace("C-q:Quit", "")
            .replace("Ctrl+Q: Quit", "")
            .replace("Ctrl+Q:Quit", "");
        assert!(!stripped.contains("q:Quit"), "width {width}: {hint}");
        assert!(!hint.to_lowercase().contains("error"), "{hint}");
        assert!(!hint.to_lowercase().contains("warning"), "{hint}");
    }
}

#[test]
fn footer_hint_wide_mentions_mouse() {
    let app = chrome_app();
    let hint = app.footer_hint(160);
    assert!(hint.contains("mouse"), "{hint}");
}

#[test]
fn draw_smoke_header_is_three_rows() {
    let mut app = chrome_app();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal
        .draw(|f| {
            app.draw(f);
            assert_eq!(f.area().height, 24);
        })
        .expect("draw");
}

#[test]
fn draw_help_modal_does_not_panic() {
    let mut app = chrome_app();
    app.show_help = true;
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.draw(|f| app.draw(f)).expect("draw help");
}

#[test]
fn all_toast_levels_render() {
    let mut app = chrome_app();
    app.push_toast(ToastLevel::Success, "ok");
    app.push_toast(ToastLevel::Info, "info");
    app.push_toast(ToastLevel::Warning, "warn");
    app.push_toast(ToastLevel::Error, "err");
    assert_eq!(app.toasts.len(), 4);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.draw(|f| app.draw(f)).expect("draw toasts");
}

#[test]
fn warning_toast_from_theme_status_does_not_panic_draw() {
    let mut app = App::new(
        AppTheme::default(),
        "default".to_string(),
        Some("theme 'foo.yml' declares version 99 (expected 1); using built-in defaults".into()),
        None,
        None,
    );
    app.push_toast(ToastLevel::Warning, app.theme_status.clone().unwrap());
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.draw(|f| app.draw(f)).expect("draw toast");
}

#[test]
fn p_without_template_toasts_warning() {
    let mut app = chrome_app();
    send_key(&mut app, KeyCode::Char('p'), KeyModifiers::NONE);
    assert!(
        app.toasts
            .iter()
            .any(|t| t.message.contains("No template open"))
    );
}

#[test]
fn shift_e_without_template_toasts_warning() {
    let mut app = chrome_app();
    send_key(&mut app, KeyCode::Char('E'), KeyModifiers::SHIFT);
    assert!(
        app.toasts
            .iter()
            .any(|t| t.message.contains("No template open"))
    );
}

#[test]
fn f3_without_template_toasts_warning() {
    let mut app = chrome_app();
    send_key(&mut app, KeyCode::F(3), KeyModifiers::NONE);
    assert!(app.modal.is_none());
    assert!(
        app.toasts
            .iter()
            .any(|t| t.message.contains("No template open"))
    );
}

#[test]
fn f3_on_minimal_template_passes() {
    let mut app = app_with_template();
    send_key(&mut app, KeyCode::F(3), KeyModifiers::NONE);
    assert!(app.modal.is_none());
    assert!(
        app.toasts
            .iter()
            .any(|t| t.message.contains("Validation passed"))
    );
}

#[test]
fn f3_on_transactional_starter_is_clean() {
    let t = crate::model::Template::starter(crate::starters::StarterKind::Transactional, "receipt");
    let mut app = App::new(
        AppTheme::default(),
        "default".to_string(),
        None,
        Some(t),
        None,
    );
    send_key(&mut app, KeyCode::F(3), KeyModifiers::NONE);
    assert!(app.modal.is_none());
    assert!(
        app.toasts
            .iter()
            .any(|t| t.message.contains("Validation passed"))
    );
    assert!(
        !app.toasts
            .iter()
            .any(|t| matches!(t.level, ToastLevel::Warning))
    );
}

#[test]
fn f3_on_invalid_template_opens_modal() {
    let mut t = crate::model::Template::minimal();
    t.name.clear();
    let mut app = App::new(
        AppTheme::default(),
        "default".to_string(),
        None,
        Some(t),
        None,
    );
    send_key(&mut app, KeyCode::F(3), KeyModifiers::NONE);
    match &app.modal {
        Some(Modal::ValidationErrors { errors, .. }) => {
            assert!(errors.iter().any(|e| e.contains("name is empty")));
        }
        other => panic!("expected ValidationErrors, got {other:?}"),
    }
}

#[test]
fn dirty_ctrl_q_opens_confirm() {
    let mut app = app_with_template();
    if let Some(t) = app.template.as_mut() {
        t.subject = "changed".to_string();
    }
    app.mark_dirty_if_changed();
    assert!(app.dirty);
    send_key(&mut app, KeyCode::Char('q'), KeyModifiers::CONTROL);
    assert!(!app.should_quit);
    match &app.modal {
        Some(Modal::ConfirmPrompt { .. }) => {}
        other => panic!("expected ConfirmPrompt, got {other:?}"),
    }
    send_key(&mut app, KeyCode::Char('y'), KeyModifiers::NONE);
    assert!(app.should_quit);
}

#[test]
fn save_without_path_opens_prompt() {
    let mut app = app_with_template();
    send_key(&mut app, KeyCode::Char('s'), KeyModifiers::NONE);
    match &app.modal {
        Some(Modal::SavePrompt { path }) => assert_eq!(path, "template.json"),
        other => panic!("expected SavePrompt, got {other:?}"),
    }
}

#[test]
fn footer_marks_dirty() {
    let mut app = app_with_template();
    app.dirty = true;
    let hint = app.footer_hint(80);
    assert!(hint.starts_with("*  "), "{hint}");
    assert!(hint.contains("F3"), "{hint}");
}

#[test]
fn tree_has_head_brand_body() {
    let app = app_with_template();
    let rows = app.tree_rows();
    assert!(matches!(rows[0].id, super::tree::TreeId::Head));
    assert!(matches!(rows[1].id, super::tree::TreeId::Brand));
    assert!(matches!(rows[2].id, super::tree::TreeId::Body));
}

#[test]
fn j_k_moves_tree_selection() {
    let mut app = app_with_template();
    assert_eq!(app.selected_row, 0);
    send_key(&mut app, KeyCode::Char('j'), KeyModifiers::NONE);
    assert_eq!(app.selected_row, 1);
    send_key(&mut app, KeyCode::Char('k'), KeyModifiers::NONE);
    assert_eq!(app.selected_row, 0);
    send_key(&mut app, KeyCode::Char('G'), KeyModifiers::SHIFT);
    assert_eq!(app.selected_row, app.tree_rows().len() - 1);
    send_key(&mut app, KeyCode::Char('g'), KeyModifiers::NONE);
    assert_eq!(app.selected_row, 0);
}

#[test]
fn tab_toggles_pane_when_details_visible() {
    let mut app = app_with_template();
    app.details_visible = true;
    assert_eq!(app.pane, super::tree::PaneFocus::Structure);
    send_key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(app.pane, super::tree::PaneFocus::Details);
    send_key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(app.pane, super::tree::PaneFocus::Structure);
}

#[test]
fn tab_noop_when_details_hidden() {
    let mut app = app_with_template();
    app.details_visible = false;
    send_key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(app.pane, super::tree::PaneFocus::Structure);
}

#[test]
fn enter_opens_formedit() {
    let mut app = app_with_template();
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    match &app.modal {
        Some(Modal::FormEdit { state, .. }) => assert_eq!(state.form.title, "mj-head"),
        other => panic!("expected FormEdit, got {other:?}"),
    }
}

#[test]
fn h_collapses_body_with_children() {
    use crate::model::{BodyNode, EmailHeader};
    let mut t = crate::model::Template::minimal();
    t.body.nodes.push(BodyNode::EmailHeader(EmailHeader {
        logo_src: String::new(),
        logo_alt: String::new(),
        logo_href: None,
        logo_width: "160px".into(),
        background_color: None,
    }));
    let mut app = App::new(
        AppTheme::default(),
        "default".to_string(),
        None,
        Some(t),
        None,
    );
    assert!(
        app.tree_rows()
            .iter()
            .any(|r| r.label.contains("email-header"))
    );
    // BODY is row 2
    app.selected_row = 2;
    send_key(&mut app, KeyCode::Char('h'), KeyModifiers::NONE);
    assert!(
        !app.tree_rows()
            .iter()
            .any(|r| r.label.contains("email-header"))
    );
    send_key(&mut app, KeyCode::Char('l'), KeyModifiers::NONE);
    assert!(
        app.tree_rows()
            .iter()
            .any(|r| r.label.contains("email-header"))
    );
}

#[test]
fn j_in_details_does_not_move_tree() {
    let mut app = app_with_template();
    app.details_visible = true;
    app.pane = super::tree::PaneFocus::Details;
    send_key(&mut app, KeyCode::Char('j'), KeyModifiers::NONE);
    assert_eq!(app.selected_row, 0);
}

#[test]
fn shift_tab_toggles_pane() {
    let mut app = app_with_template();
    app.details_visible = true;
    send_key(&mut app, KeyCode::BackTab, KeyModifiers::SHIFT);
    assert_eq!(app.pane, super::tree::PaneFocus::Details);
}

#[test]
fn details_head_and_brand_are_labeled() {
    let app = app_with_template();
    let rows = app.tree_rows();
    let head = super::details::details_lines(app.template.as_ref(), rows.get(0), 40);
    assert!(head.iter().any(|l| l.starts_with("subject:")));
    assert!(head.iter().any(|l| l.starts_with("json_ld:")));
    let brand = super::details::details_lines(app.template.as_ref(), rows.get(1), 40);
    assert!(brand.iter().any(|l| l.contains("button_background")));
    assert!(brand.iter().any(|l| l.contains('#')));
}

#[test]
fn draw_template_master_detail_does_not_panic() {
    let mut app = app_with_template();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.draw(|f| app.draw(f)).expect("draw");
    assert!(app.details_visible);
    let backend = TestBackend::new(40, 16);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.draw(|f| app.draw(f)).expect("draw narrow");
    assert!(!app.details_visible);
}

fn app_with_one_column() -> App {
    use crate::model::{BodyNode, MjColumn, MjSection, SectionChild};
    let mut t = crate::model::Template::minimal();
    t.body.nodes.push(BodyNode::MjSection(MjSection {
        background_color: None,
        padding: None,
        full_width: false,
        children: vec![SectionChild::MjColumn(MjColumn {
            width: Some("100%".into()),
            background_color: None,
            padding: None,
            inner_background_color: None,
            components: Vec::new(),
            ..Default::default()
        })],
        ..Default::default()
    }));
    App::new(
        AppTheme::default(),
        "default".to_string(),
        None,
        Some(t),
        None,
    )
}

fn form_state(app: &App) -> &super::editform::EditFormState {
    match &app.modal {
        Some(Modal::FormEdit { state, .. }) => state,
        other => panic!("expected FormEdit, got {other:?}"),
    }
}

fn form_state_mut(app: &mut App) -> &mut super::editform::EditFormState {
    match &mut app.modal {
        Some(Modal::FormEdit { state, .. }) => state,
        other => panic!("expected FormEdit, got {other:?}"),
    }
}

#[test]
fn form_tab_moves_field() {
    let mut app = app_with_template();
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(form_state(&app).focused_field, 0);
    send_key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(form_state(&app).focused_field, 1);
    send_key(&mut app, KeyCode::BackTab, KeyModifiers::SHIFT);
    assert_eq!(form_state(&app).focused_field, 0);
}

#[test]
fn form_enum_cycles_css_inline() {
    let mut app = app_with_template();
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    let idx = form_state(&app)
        .field_index("css_inline")
        .expect("css_inline field");
    form_state_mut(&mut app).focused_field = idx;
    assert_eq!(form_state(&app).get("css_inline"), "false");
    send_key(&mut app, KeyCode::Right, KeyModifiers::NONE);
    assert_eq!(form_state(&app).get("css_inline"), "true");
    send_key(&mut app, KeyCode::Left, KeyModifiers::NONE);
    assert_eq!(form_state(&app).get("css_inline"), "false");
}

#[test]
fn brand_edit_saves() {
    let mut app = app_with_template();
    app.selected_row = 1;
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(form_state(&app).form.title, "brand");
    form_state_mut(&mut app).set("font_family", "Raleway, Arial, sans-serif");
    send_key(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(app.modal.is_none());
    assert_eq!(
        app.template.as_ref().unwrap().brand.font_family,
        "Raleway, Arial, sans-serif"
    );
}

#[test]
fn add_google_font_row() {
    let mut app = app_with_template();
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    let idx = form_state(&app).field_index("fonts").expect("fonts field");
    form_state_mut(&mut app).focused_field = idx;
    send_key(&mut app, KeyCode::Char('A'), KeyModifiers::SHIFT);
    let fonts = form_state(&app)
        .sub_state
        .get("fonts")
        .expect("fonts collection");
    assert_eq!(fonts.len(), 1);
    assert_eq!(fonts[0].get("name"), "Raleway");
    assert!(fonts[0].get("href").contains("fonts.googleapis.com"));
}

#[test]
fn json_ld_textarea_accepts_input() {
    let mut app = app_with_template();
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    let idx = form_state(&app)
        .field_index("json_ld")
        .expect("json_ld field");
    form_state_mut(&mut app).focused_field = idx;
    if let Some(Modal::FormEdit { cursor_pos, .. }) = app.modal.as_mut() {
        *cursor_pos = 0;
    }
    send_key(&mut app, KeyCode::Char('{'), KeyModifiers::NONE);
    send_key(&mut app, KeyCode::Char('}'), KeyModifiers::NONE);
    assert_eq!(form_state(&app).get("json_ld"), "{}");
}

#[test]
fn delete_guards_head_brand_body() {
    let mut app = app_with_template();
    app.selected_row = 0;
    send_key(&mut app, KeyCode::Char('d'), KeyModifiers::NONE);
    app.selected_row = 1;
    send_key(&mut app, KeyCode::Char('d'), KeyModifiers::NONE);
    app.selected_row = 2;
    send_key(&mut app, KeyCode::Char('d'), KeyModifiers::NONE);
    let guarded = app
        .toasts
        .iter()
        .filter(|t| t.message.contains("Cannot delete this row."))
        .count();
    assert_eq!(guarded, 3);
    assert!(app.template.is_some());
}

#[test]
fn c_splits_one_column_fifty_fifty() {
    let mut app = app_with_one_column();
    let idx = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("mj-section"))
        .expect("section row");
    app.selected_row = idx;
    send_key(&mut app, KeyCode::Char('C'), KeyModifiers::SHIFT);
    let crate::model::BodyNode::MjSection(s) = &app.template.as_ref().unwrap().body.nodes[0] else {
        panic!("expected mj-section");
    };
    assert_eq!(s.children.len(), 2);
    let widths: Vec<_> = s
        .children
        .iter()
        .map(|c| match c {
            crate::model::SectionChild::MjColumn(col) => col.width.clone().unwrap(),
            _ => panic!("expected column"),
        })
        .collect();
    assert_eq!(widths, vec!["50%", "50%"]);
}

#[test]
fn v_refuses_last_column() {
    let mut app = app_with_one_column();
    let idx = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("mj-section"))
        .expect("section row");
    app.selected_row = idx;
    send_key(&mut app, KeyCode::Char('V'), KeyModifiers::SHIFT);
    assert!(
        app.toasts
            .iter()
            .any(|t| t.message.contains("A section needs at least one column"))
    );
    let crate::model::BodyNode::MjSection(s) = &app.template.as_ref().unwrap().body.nodes[0] else {
        panic!("expected mj-section");
    };
    assert_eq!(s.children.len(), 1);
}

#[test]
fn draw_formedit_does_not_panic() {
    let mut app = app_with_template();
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.draw(|f| app.draw(f)).expect("draw form");
}

fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
    let buf = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            out.push_str(buf[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

#[test]
fn formedit_padding_shows_expected_units() {
    let mut app = app_with_one_column();
    let idx = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("mj-column"))
        .expect("column row");
    app.selected_row = idx;
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    match &app.modal {
        Some(Modal::FormEdit { state, .. }) => {
            let pad = state
                .form
                .fields
                .iter()
                .find(|f| f.id == "padding")
                .expect("padding field");
            assert_eq!(pad.hint, Some(crate::padding::HINT));
            assert_eq!(pad.placeholder, Some(crate::padding::PLACEHOLDER));
        }
        other => panic!("expected FormEdit, got {other:?}"),
    }
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.draw(|f| app.draw(f)).expect("draw form");
    let text = buffer_text(&terminal);
    assert!(text.contains("1-4"), "{text}");
    assert!(text.contains("px"), "{text}");
    assert!(text.contains("10px"), "{text}");
}

#[test]
fn formedit_shows_brand_placeholders_on_text() {
    let app = app_with_mj_text_form();
    let brand = app.template.as_ref().unwrap().brand.clone();
    match &app.modal {
        Some(Modal::FormEdit { state, .. }) => {
            let color = state
                .form
                .fields
                .iter()
                .find(|f| f.id == "color")
                .expect("color");
            assert_eq!(
                color.resolved_placeholder(Some(&brand)).as_deref(),
                Some(brand.text_color.as_str())
            );
            let font = state
                .form
                .fields
                .iter()
                .find(|f| f.id == "font_family")
                .expect("font_family");
            assert_eq!(
                font.resolved_placeholder(Some(&brand)).as_deref(),
                Some(brand.font_family.as_str())
            );
        }
        other => panic!("expected FormEdit, got {other:?}"),
    }
}

#[test]
fn formedit_shows_brand_placeholders_on_button() {
    let mut app = app_with_one_column();
    let col = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("mj-column"))
        .expect("column row");
    app.selected_row = col;
    app.insert_kind(super::component_kind::ComponentKind::MjButton);
    let btn = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("mj-button"))
        .expect("mj-button row");
    app.selected_row = btn;
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    let brand = app.template.as_ref().unwrap().brand.clone();
    match &app.modal {
        Some(Modal::FormEdit { state, .. }) => {
            let bg = state
                .form
                .fields
                .iter()
                .find(|f| f.id == "background_color")
                .expect("background_color");
            assert_eq!(
                bg.resolved_placeholder(Some(&brand)).as_deref(),
                Some(brand.button_background.as_str())
            );
            let color = state
                .form
                .fields
                .iter()
                .find(|f| f.id == "color")
                .expect("color");
            assert_eq!(
                color.resolved_placeholder(Some(&brand)).as_deref(),
                Some(brand.button_color.as_str())
            );
        }
        other => panic!("expected FormEdit, got {other:?}"),
    }
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.draw(|f| app.draw(f)).expect("draw form");
    let text = buffer_text(&terminal);
    assert!(text.contains(&brand.button_background), "{text}");
    assert!(text.contains("empty uses brand"), "{text}");
}

#[test]
fn formedit_body_background_placeholder_is_brand() {
    let mut app = app_with_template();
    app.selected_row = 2;
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(form_state(&app).form.title, "mj-body");
    let brand = app.template.as_ref().unwrap().brand.clone();
    let bg = form_state(&app)
        .form
        .fields
        .iter()
        .find(|f| f.id == "background_color")
        .expect("background_color");
    assert_eq!(
        bg.resolved_placeholder(Some(&brand)).as_deref(),
        Some(brand.background_color.as_str())
    );
}

#[test]
fn form_save_normalizes_unitless_padding() {
    let mut app = app_with_one_column();
    let idx = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("mj-column"))
        .expect("column row");
    app.selected_row = idx;
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    form_state_mut(&mut app).set("padding", "12 10 12 10");
    send_key(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(app.modal.is_none());
    let crate::model::BodyNode::MjSection(s) = &app.template.as_ref().unwrap().body.nodes[0] else {
        panic!("expected mj-section");
    };
    let crate::model::SectionChild::MjColumn(col) = &s.children[0] else {
        panic!("expected column");
    };
    assert_eq!(col.padding.as_deref(), Some("12px 10px 12px 10px"));
}

#[test]
fn form_save_rejects_invalid_padding() {
    let mut app = app_with_one_column();
    let idx = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("mj-column"))
        .expect("column row");
    app.selected_row = idx;
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    form_state_mut(&mut app).set("padding", "10em");
    send_key(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(matches!(app.modal, Some(Modal::FormEdit { .. })));
    assert!(app.toasts.iter().any(|t| t.message.contains("padding")));
}

#[test]
fn section_label_saves_and_shows_in_tree() {
    let mut app = app_with_one_column();
    let idx = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("mj-section"))
        .expect("section row");
    app.selected_row = idx;
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(form_state(&app).form.fields[0].id, "label");
    form_state_mut(&mut app).set("label", "footer");
    send_key(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(app.modal.is_none());
    match &app.template.as_ref().unwrap().body.nodes[0] {
        crate::model::BodyNode::MjSection(s) => {
            assert_eq!(s.label.as_deref(), Some("footer"));
        }
        other => panic!("expected section, got {other:?}"),
    }
    let labels: Vec<_> = app.tree_rows().iter().map(|r| r.label.clone()).collect();
    assert!(
        labels.iter().any(|l| l.contains("mj-section  footer")),
        "{labels:?}"
    );
}

#[test]
fn border_sides_checkboxes_toggle_and_save() {
    let mut app = app_with_one_column();
    let idx = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("mj-section"))
        .expect("section row");
    app.selected_row = idx;
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    let field = form_state(&app)
        .field_index("border_sides")
        .expect("border_sides field");
    form_state_mut(&mut app).focused_field = field;
    form_state_mut(&mut app).checkbox_cursor = 0;
    assert_eq!(form_state(&app).get("border_sides"), "all");
    send_key(&mut app, KeyCode::Right, KeyModifiers::NONE);
    send_key(&mut app, KeyCode::Char(' '), KeyModifiers::NONE);
    assert_eq!(form_state(&app).get("border_sides"), "top");
    send_key(&mut app, KeyCode::Right, KeyModifiers::NONE);
    send_key(&mut app, KeyCode::Right, KeyModifiers::NONE);
    send_key(&mut app, KeyCode::Char(' '), KeyModifiers::NONE);
    assert_eq!(form_state(&app).get("border_sides"), "top,bottom");
    form_state_mut(&mut app).set("border", "1px solid #000");
    send_key(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(app.modal.is_none());
    match &app.template.as_ref().unwrap().body.nodes[0] {
        crate::model::BodyNode::MjSection(s) => {
            assert_eq!(s.border.as_deref(), Some("1px solid #000"));
            assert_eq!(s.border_sides.as_deref(), Some("top,bottom"));
        }
        other => panic!("expected section, got {other:?}"),
    }
}

#[test]
fn insert_wraps_leaf_on_empty_body() {
    let mut app = app_with_template();
    app.selected_row = 2;
    send_key(&mut app, KeyCode::Char('/'), KeyModifiers::NONE);
    match &app.modal {
        Some(Modal::ComponentPicker { .. }) => {}
        other => panic!("expected ComponentPicker, got {other:?}"),
    }
    for c in "mj-text".chars() {
        send_key(&mut app, KeyCode::Char(c), KeyModifiers::NONE);
    }
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert!(
        app.toasts
            .iter()
            .any(|t| t.message.contains("Wrapped in mj-section"))
    );
    let nodes = &app.template.as_ref().unwrap().body.nodes;
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        crate::model::BodyNode::MjSection(s) => {
            let crate::model::SectionChild::MjColumn(c) = &s.children[0] else {
                panic!("expected column");
            };
            assert!(matches!(
                c.components[0],
                crate::model::ColumnChild::MjText(_)
            ));
        }
        other => panic!("expected section wrap, got {other:?}"),
    }
}

#[test]
fn insert_illegal_on_head_toasts() {
    let mut app = app_with_template();
    app.selected_row = 0;
    app.insert_kind(super::component_kind::ComponentKind::MjText);
    assert!(
        app.toasts
            .iter()
            .any(|t| t.message.contains("Cannot insert mj-text here"))
    );
    assert!(app.template.as_ref().unwrap().body.nodes.is_empty());
}

#[test]
fn insert_navbar_on_column_and_link_under_navbar() {
    let mut app = app_with_one_column();
    let col = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("mj-column"))
        .expect("column row");
    app.selected_row = col;
    app.insert_kind(super::component_kind::ComponentKind::MjNavbar);
    let crate::model::BodyNode::MjSection(s) = &app.template.as_ref().unwrap().body.nodes[0] else {
        panic!("expected section");
    };
    let crate::model::SectionChild::MjColumn(c) = &s.children[0] else {
        panic!("expected column");
    };
    assert!(matches!(
        c.components[0],
        crate::model::ColumnChild::MjNavbar(_)
    ));
    let nav = app
        .tree_rows()
        .iter()
        .position(|r| r.label == "mj-navbar")
        .expect("navbar row");
    app.selected_row = nav;
    app.insert_kind(super::component_kind::ComponentKind::MjNavbarLink);
    let crate::model::BodyNode::MjSection(s) = &app.template.as_ref().unwrap().body.nodes[0] else {
        panic!("expected section");
    };
    let crate::model::SectionChild::MjColumn(c) = &s.children[0] else {
        panic!("expected column");
    };
    let crate::model::ColumnChild::MjNavbar(n) = &c.components[0] else {
        panic!("expected navbar");
    };
    assert_eq!(n.links.len(), 1);
    assert!(app.tree_rows().iter().any(|r| r.label == "mj-navbar-link"));
}

#[test]
fn insert_navbar_link_on_column_toasts() {
    let mut app = app_with_one_column();
    let col = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("mj-column"))
        .expect("column row");
    app.selected_row = col;
    app.insert_kind(super::component_kind::ComponentKind::MjNavbarLink);
    assert!(
        app.toasts
            .iter()
            .any(|t| t.message.contains("Cannot insert mj-navbar-link here"))
    );
}

#[test]
fn insert_navbar_link_on_section_toasts() {
    let mut app = app_with_one_column();
    let idx = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("mj-section"))
        .expect("section row");
    app.selected_row = idx;
    app.insert_kind(super::component_kind::ComponentKind::MjNavbarLink);
    assert!(
        app.toasts
            .iter()
            .any(|t| t.message.contains("Cannot insert mj-navbar-link here"))
    );
}

#[test]
fn image_picker_writes_relative_path() {
    let dir = std::env::temp_dir().join(format!("dd_emailforge_5c_img_{}", std::process::id()));
    let images = dir.join("images");
    std::fs::create_dir_all(&images).expect("mkdir images");
    std::fs::write(images.join("hero.png"), b"png").expect("write png");
    let mut app = app_with_template();
    app.path = Some(dir.join("template.json"));
    app.selected_row = 2;
    send_key(&mut app, KeyCode::Char('/'), KeyModifiers::NONE);
    for c in "email-header".chars() {
        send_key(&mut app, KeyCode::Char(c), KeyModifiers::NONE);
    }
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    let idx = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("email-header"))
        .expect("header row");
    app.selected_row = idx;
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    match &app.modal {
        Some(Modal::FormEdit { state, .. }) => assert_eq!(state.form.fields[0].id, "logo_src"),
        other => panic!("expected FormEdit, got {other:?}"),
    }
    send_key(&mut app, KeyCode::Char('p'), KeyModifiers::CONTROL);
    match &app.modal {
        Some(Modal::ImagePicker { .. }) => {}
        other => panic!("expected ImagePicker, got {other:?}"),
    }
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    match &app.modal {
        Some(Modal::FormEdit { state, .. }) => {
            assert_eq!(state.get("logo_src"), "images/hero.png");
        }
        other => panic!("expected FormEdit after pick, got {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn click_ascii_selects_body_node() {
    let mut app = app_with_one_column();
    app.selected_row = 2;
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.draw(|f| app.draw(f)).expect("draw");
    let (rect, id) = app
        .details_hit_areas
        .iter()
        .rev()
        .find(|(r, id)| {
            r.width > 2
                && matches!(id, super::tree::TreeId::Path(p) if matches!(p.as_slice(), [super::tree::Step::BodyNode(_)]))
        })
        .cloned()
        .expect("body node hit");
    send_mouse(
        &mut app,
        MouseEventKind::Down(MouseButton::Left),
        rect.x,
        rect.y,
    );
    assert_eq!(app.selected_tree_id(), Some(id));
}

#[test]
fn click_blueprint_selects_nested_component() {
    let mut app = app_with_one_column();
    let col = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("mj-column"))
        .expect("column row");
    app.selected_row = col;
    app.insert_kind(super::component_kind::ComponentKind::MjText);
    app.selected_row = 2;
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.draw(|f| app.draw(f)).expect("draw");
    let (rect, id) = app
        .details_hit_areas
        .iter()
        .rev()
        .find(|(_, id)| {
            matches!(
                id,
                super::tree::TreeId::Path(p) if matches!(p.last(), Some(super::tree::Step::ColComp(_)))
            )
        })
        .cloned()
        .expect("component hit");
    send_mouse(
        &mut app,
        MouseEventKind::Down(MouseButton::Left),
        rect.x,
        rect.y,
    );
    assert_eq!(app.selected_tree_id(), Some(id));
    let label = app
        .tree_rows()
        .get(app.selected_row)
        .map(|r| r.label.clone())
        .unwrap_or_default();
    assert!(label.contains("mj-text"), "{label}");
}

#[test]
fn footer_medium_mentions_insert() {
    let app = app_with_template();
    let hint = app.footer_hint(80);
    assert!(
        hint.contains("/: Insert") || hint.contains("/:Insert"),
        "{hint}"
    );
}

fn app_with_mj_text_form() -> App {
    let mut app = app_with_one_column();
    let col = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("mj-column"))
        .expect("column row");
    app.selected_row = col;
    app.insert_kind(super::component_kind::ComponentKind::MjText);
    let text = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("mj-text"))
        .expect("mj-text row");
    app.selected_row = text;
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert!(matches!(app.modal, Some(Modal::FormEdit { .. })));
    assert_eq!(form_state(&app).focused().map(|f| f.id), Some("content"));
    app
}

fn mj_text_content(app: &App) -> String {
    let crate::model::BodyNode::MjSection(s) = &app.template.as_ref().unwrap().body.nodes[0] else {
        panic!("expected mj-section");
    };
    let crate::model::SectionChild::MjColumn(col) = &s.children[0] else {
        panic!("expected column");
    };
    match &col.components[0] {
        crate::model::ColumnChild::MjText(t) => t.content.clone(),
        other => panic!("expected mj-text, got {other:?}"),
    }
}

#[test]
fn textarea_expand_ctrl_e_opens_and_esc_returns_to_form() {
    let mut app = app_with_mj_text_form();
    send_key(&mut app, KeyCode::Char('e'), KeyModifiers::CONTROL);
    assert!(app.form_textarea_expanded);
    assert!(matches!(app.modal, Some(Modal::FormEdit { .. })));

    send_key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!app.form_textarea_expanded);
    assert!(matches!(app.modal, Some(Modal::FormEdit { .. })));
    assert_eq!(form_state(&app).focused().map(|f| f.id), Some("content"));
}

#[test]
fn textarea_expand_ctrl_e_on_non_textarea_is_noop() {
    let mut app = app_with_template();
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(form_state(&app).form.fields[0].id, "subject");
    send_key(&mut app, KeyCode::Char('e'), KeyModifiers::CONTROL);
    assert!(!app.form_textarea_expanded);
    assert!(matches!(app.modal, Some(Modal::FormEdit { .. })));
}

#[test]
fn textarea_expand_esc_from_form_still_closes() {
    let mut app = app_with_mj_text_form();
    send_key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert!(app.modal.is_none());
    assert!(!app.form_textarea_expanded);
}

#[test]
fn textarea_expand_edits_then_save_commits_copy() {
    let mut app = app_with_mj_text_form();
    send_key(&mut app, KeyCode::Char('e'), KeyModifiers::CONTROL);
    assert!(app.form_textarea_expanded);
    send_key(&mut app, KeyCode::Char('Z'), KeyModifiers::NONE);
    send_key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!app.form_textarea_expanded);
    assert!(form_state(&app).get("content").ends_with('Z'));
    send_key(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(app.modal.is_none());
    assert!(mj_text_content(&app).ends_with('Z'));
}

#[test]
fn textarea_expand_ctrl_s_saves_from_expanded_view() {
    let mut app = app_with_mj_text_form();
    send_key(&mut app, KeyCode::Char('e'), KeyModifiers::CONTROL);
    send_key(&mut app, KeyCode::Char('Q'), KeyModifiers::NONE);
    send_key(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(app.modal.is_none());
    assert!(!app.form_textarea_expanded);
    assert!(mj_text_content(&app).ends_with('Q'));
}

#[test]
fn textarea_expand_tab_does_not_leave_field() {
    let mut app = app_with_mj_text_form();
    send_key(&mut app, KeyCode::Char('e'), KeyModifiers::CONTROL);
    send_key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    assert!(app.form_textarea_expanded);
    assert_eq!(form_state(&app).focused().map(|f| f.id), Some("content"));
}

#[test]
fn textarea_expand_click_opens_from_hit_target() {
    let mut app = app_with_mj_text_form();
    let copy_idx = form_state(&app)
        .form
        .fields
        .iter()
        .position(|f| f.id == "content")
        .expect("content field");
    app.form_expand_hits.borrow_mut().push((
        copy_idx,
        Rect {
            x: 40,
            y: 10,
            width: 8,
            height: 1,
        },
    ));
    send_mouse(&mut app, MouseEventKind::Down(MouseButton::Left), 42, 10);
    assert!(app.form_textarea_expanded);
    assert_eq!(form_state(&app).focused().map(|f| f.id), Some("content"));
}

#[test]
fn textarea_home_end_move_cursor() {
    let mut app = app_with_mj_text_form();
    form_state_mut(&mut app).set("content", "hello\nworld");
    if let Some(Modal::FormEdit { cursor_pos, .. }) = app.modal.as_mut() {
        *cursor_pos = 8;
    }
    send_key(&mut app, KeyCode::Home, KeyModifiers::NONE);
    match &app.modal {
        Some(Modal::FormEdit { cursor_pos, .. }) => assert_eq!(*cursor_pos, 6),
        other => panic!("expected FormEdit, got {other:?}"),
    }
    send_key(&mut app, KeyCode::End, KeyModifiers::NONE);
    match &app.modal {
        Some(Modal::FormEdit { cursor_pos, .. }) => assert_eq!(*cursor_pos, 11),
        other => panic!("expected FormEdit, got {other:?}"),
    }
    send_key(&mut app, KeyCode::Home, KeyModifiers::CONTROL);
    match &app.modal {
        Some(Modal::FormEdit { cursor_pos, .. }) => assert_eq!(*cursor_pos, 0),
        other => panic!("expected FormEdit, got {other:?}"),
    }
    send_key(&mut app, KeyCode::End, KeyModifiers::CONTROL);
    match &app.modal {
        Some(Modal::FormEdit { cursor_pos, .. }) => assert_eq!(*cursor_pos, 11),
        other => panic!("expected FormEdit, got {other:?}"),
    }
}

#[test]
fn textarea_click_places_cursor() {
    let mut app = app_with_mj_text_form();
    form_state_mut(&mut app).set("content", "abcdefghij");
    let field_idx = form_state(&app)
        .form
        .fields
        .iter()
        .position(|f| f.id == "content")
        .expect("content field");
    app.form_textarea_hits
        .borrow_mut()
        .push(super::form_textarea::TextareaHit {
            rect: Rect {
                x: 10,
                y: 5,
                width: 20,
                height: 4,
            },
            field_idx,
            first_visible_row: 0,
            wrap_width: 20,
        });
    send_mouse(&mut app, MouseEventKind::Down(MouseButton::Left), 13, 5);
    match &app.modal {
        Some(Modal::FormEdit { cursor_pos, .. }) => assert_eq!(*cursor_pos, 3),
        other => panic!("expected FormEdit, got {other:?}"),
    }
}

#[test]
fn expanded_textarea_click_places_cursor() {
    let mut app = app_with_mj_text_form();
    send_key(&mut app, KeyCode::Char('e'), KeyModifiers::CONTROL);
    assert!(app.form_textarea_expanded);
    form_state_mut(&mut app).set("content", "abcdefghij");
    let field_idx = form_state(&app)
        .form
        .fields
        .iter()
        .position(|f| f.id == "content")
        .expect("content field");
    app.form_textarea_hits
        .borrow_mut()
        .push(super::form_textarea::TextareaHit {
            rect: Rect {
                x: 4,
                y: 4,
                width: 40,
                height: 10,
            },
            field_idx,
            first_visible_row: 0,
            wrap_width: 40,
        });
    send_mouse(&mut app, MouseEventKind::Down(MouseButton::Left), 6, 4);
    match &app.modal {
        Some(Modal::FormEdit { cursor_pos, .. }) => assert_eq!(*cursor_pos, 2),
        other => panic!("expected FormEdit, got {other:?}"),
    }
    assert!(app.form_textarea_expanded);
}

#[test]
fn f1_from_expanded_textarea_keeps_form() {
    let mut app = app_with_mj_text_form();
    send_key(&mut app, KeyCode::Char('e'), KeyModifiers::CONTROL);
    send_key(&mut app, KeyCode::F(1), KeyModifiers::NONE);
    assert!(app.show_help);
    assert!(matches!(app.modal, Some(Modal::FormEdit { .. })));
    assert!(app.form_textarea_expanded);
    send_key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!app.show_help);
    assert!(app.form_textarea_expanded);
    assert!(matches!(app.modal, Some(Modal::FormEdit { .. })));
}
