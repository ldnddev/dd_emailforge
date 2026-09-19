//! Capture live TUI frames as HTML for `docs/capture.sh`.
//!
//! Run via that script (or `EMAILFORGE_TUTORIAL_SHOTS=… cargo test --offline
//! capture_tutorial_frames -- --ignored --nocapture`).
use super::*;
use crate::model::Template;
use crate::starters::StarterKind;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::CellDiffOption;
use ratatui::style::Color;
use std::fs;
use std::path::Path;

const COLS: u16 = 100;
const ROWS: u16 = 30;
const HEADER_COPY: &str = "600 pixels wide. Infinite opinions.";

#[test]
#[ignore = "writes tutorial HTML frames; run docs/capture.sh"]
fn capture_tutorial_frames() {
    let dir = std::env::var("EMAILFORGE_TUTORIAL_SHOTS")
        .unwrap_or_else(|_| format!("{}/docs/images", env!("CARGO_MANIFEST_DIR")));
    let dir = Path::new(&dir);
    fs::create_dir_all(dir).expect("mkdir shot dir");

    write_frame(dir, "tui-empty", &mut empty_app());
    write_frame(dir, "tui-welcome", &mut welcome_app());
    write_frame(dir, "tui-insert", &mut insert_app());
    write_frame(dir, "tui-formedit", &mut formedit_app());
    write_frame(dir, "tui-help", &mut help_app());
    write_frame(dir, "tui-selected", &mut selected_text_app());
}

fn empty_app() -> App {
    let mut app = shot_app(None);
    app.push_toast(
        ToastLevel::Info,
        "No template open. Run: dd_emailforge init <dir>",
    );
    app
}

fn welcome_app() -> App {
    let mut app = shot_app(Some(Template::starter(StarterKind::Welcome, "welcome")));
    app.selected_row = 2; // [BODY]
    app
}

fn insert_app() -> App {
    let mut app = shot_app(Some(Template::starter(StarterKind::Welcome, "welcome")));
    let idx = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("mj-column"))
        .expect("column");
    app.selected_row = idx;
    send_key(&mut app, KeyCode::Char('/'), KeyModifiers::NONE);
    app
}

fn formedit_app() -> App {
    let mut app = shot_app(Some(Template::starter(StarterKind::Welcome, "welcome")));
    let idx = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("email-header"))
        .expect("header");
    app.selected_row = idx;
    send_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    app
}

fn help_app() -> App {
    let mut app = shot_app(Some(Template::starter(StarterKind::Welcome, "welcome")));
    send_key(&mut app, KeyCode::F(1), KeyModifiers::NONE);
    app
}

fn selected_text_app() -> App {
    let mut app = shot_app(Some(Template::starter(StarterKind::Welcome, "welcome")));
    let idx = app
        .tree_rows()
        .iter()
        .position(|r| r.label.contains("mj-text"))
        .expect("text");
    app.selected_row = idx;
    app
}

fn shot_app(template: Option<Template>) -> App {
    let (theme, source, status) = AppTheme::load();
    let mut app = App::new(theme, source, status, template, None);
    app.header_copy = HEADER_COPY.into();
    app
}

fn send_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    app.handle_event(Event::Key(KeyEvent::new(code, modifiers)))
        .expect("key");
}

fn write_frame(dir: &Path, name: &str, app: &mut App) {
    let backend = TestBackend::new(COLS, ROWS);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.draw(|f| app.draw(f)).expect("draw");
    terminal.draw(|f| app.draw(f)).expect("draw settle");
    let html = frame_html(name, terminal.backend().buffer());
    let path = dir.join(format!("_frame-{name}.html"));
    fs::write(&path, html).expect("write frame html");
}

fn frame_html(title: &str, buffer: &ratatui::buffer::Buffer) -> String {
    let area = *buffer.area();
    let mut rows = String::new();
    for y in 0..area.height {
        rows.push_str("<div class=\"row\">");
        let mut x = 0u16;
        while x < area.width {
            let cell = &buffer[(x, y)];
            if cell.diff_option == CellDiffOption::Skip {
                x += 1;
                continue;
            }
            let fg = css_color(cell.fg, "#F5F6F7");
            let bg = css_color(cell.bg, "#0F1114");
            let mut x1 = x + 1;
            while x1 < area.width {
                let next = &buffer[(x1, y)];
                if next.diff_option == CellDiffOption::Skip {
                    break;
                }
                if css_color(next.fg, "#F5F6F7") != fg || css_color(next.bg, "#0F1114") != bg {
                    break;
                }
                x1 += 1;
            }
            let mut text = String::new();
            for cx in x..x1 {
                text.push_str(buffer[(cx, y)].symbol());
            }
            let bold = if cell.modifier.contains(Modifier::BOLD) {
                " font-weight:700;"
            } else {
                ""
            };
            rows.push_str(&format!(
                "<span style=\"color:{fg};background:{bg};{bold}\">{}</span>",
                esc(&text)
            ));
            x = x1;
        }
        rows.push_str("</div>\n");
    }
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{title}</title>
<style>
  html, body {{
    margin: 0;
    padding: 0;
    background: #1a1c1f;
    width: 900px;
    height: 590px;
  }}
  .window {{
    margin: 12px;
    width: 876px;
    border: 1px solid #3a3d42;
    border-radius: 10px;
    overflow: hidden;
    box-shadow: 0 18px 40px rgba(0,0,0,.45);
    background: #0F1114;
  }}
  .titlebar {{
    height: 32px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    background: #1C1E21;
    color: #9EA3AA;
    font: 12px/1 ui-sans-serif, system-ui, sans-serif;
    border-bottom: 1px solid #3a3d42;
  }}
  .dot {{ width: 10px; height: 10px; border-radius: 50%; display: inline-block; }}
  .term {{
    font: 12px/16px "DejaVu Sans Mono", "Liberation Mono", "Noto Sans Mono", ui-monospace, monospace;
    font-variant-ligatures: none;
    letter-spacing: 0;
    padding: 6px 8px 8px;
    background: #0F1114;
  }}
  .row {{
    display: block;
    white-space: pre;
    overflow: hidden;
    height: 16px;
    line-height: 16px;
  }}
  .row span {{ white-space: pre; }}
</style>
</head>
<body>
  <div class="window">
    <div class="titlebar">
      <span class="dot" style="background:#e57373"></span>
      <span class="dot" style="background:#f5c469"></span>
      <span class="dot" style="background:#82e0aa"></span>
      <span>dd_emailforge — {title} · 100×30</span>
    </div>
    <div class="term">{rows}</div>
  </div>
</body>
</html>
"##,
        title = title,
        rows = rows
    )
}

fn css_color(c: Color, fallback: &str) -> String {
    match c {
        Color::Reset => fallback.to_string(),
        Color::Black => "#000000".into(),
        Color::Red => "#cc3333".into(),
        Color::Green => "#33cc33".into(),
        Color::Yellow => "#cccc33".into(),
        Color::Blue => "#3366cc".into(),
        Color::Magenta => "#cc33cc".into(),
        Color::Cyan => "#33cccc".into(),
        Color::Gray | Color::DarkGray => "#808080".into(),
        Color::LightRed => "#e57373".into(),
        Color::LightGreen => "#82e0aa".into(),
        Color::LightYellow => "#f5c469".into(),
        Color::LightBlue => "#64B4F5".into(),
        Color::LightMagenta => "#d980d9".into(),
        Color::LightCyan => "#80d9d9".into(),
        Color::White => "#F5F6F7".into(),
        Color::Rgb(r, g, b) => format!("#{r:02X}{g:02X}{b:02X}"),
        Color::Indexed(_) => fallback.to_string(),
    }
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}
