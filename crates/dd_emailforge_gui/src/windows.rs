//! App id and window options for the editor.
//!
//! On Wayland `app_id` is the xdg app id. On X11 it is the window class.

use gpui::{
    Bounds, Pixels, SharedString, TitlebarOptions, WindowBounds, WindowOptions, px, size,
};

pub const MAIN_APP_ID: &str = "dev.ldnddev.dd_emailforge";

pub fn main_window_options(bounds: Bounds<Pixels>) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        app_id: Some(MAIN_APP_ID.to_string()),
        titlebar: Some(TitlebarOptions {
            title: Some(SharedString::from("dd_emailforge")),
            ..Default::default()
        }),
        focus: true,
        show: true,
        ..Default::default()
    }
}

pub fn centered_bounds(cx: &gpui::App) -> Bounds<Pixels> {
    Bounds::centered(None, size(px(1280.0), px(800.0)), cx)
}
