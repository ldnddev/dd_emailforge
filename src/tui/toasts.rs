//! Ephemeral toast notifications. Four-level enum (visual-standard tokens);
//! layout matches siteforge (bottom-right, 5s, cap 4, Clear + border).

use super::*;

pub(super) const TOAST_TTL: std::time::Duration = std::time::Duration::from_secs(5);

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(super) enum ToastLevel {
    Success,
    Info,
    Warning,
    Error,
}

pub(super) struct Toast {
    pub(super) level: ToastLevel,
    pub(super) message: String,
    pub(super) shown_at: std::time::Instant,
}

impl App {
    pub(super) fn push_toast(&mut self, level: ToastLevel, message: impl Into<String>) {
        self.toasts.push(Toast {
            level,
            message: message.into(),
            shown_at: std::time::Instant::now(),
        });
        if self.toasts.len() > 4 {
            self.toasts.remove(0);
        }
    }

    pub(super) fn prune_toasts(&mut self) {
        let now = std::time::Instant::now();
        self.toasts
            .retain(|t| now.duration_since(t.shown_at) < TOAST_TTL);
    }

    pub(super) fn render_toasts(&self, frame: &mut ratatui::Frame, area: Rect) {
        let lines: Vec<super::toast_chrome::ToastLine> = self
            .toasts
            .iter()
            .map(|toast| super::toast_chrome::ToastLine {
                kind: match toast.level {
                    ToastLevel::Success => super::toast_chrome::ToastKind::Success,
                    ToastLevel::Info => super::toast_chrome::ToastKind::Info,
                    ToastLevel::Warning => super::toast_chrome::ToastKind::Warning,
                    ToastLevel::Error => super::toast_chrome::ToastKind::Error,
                },
                message: toast.message.as_str(),
            })
            .collect();
        let _ = super::toast_chrome::render_toast_stack(
            frame,
            area,
            &lines,
            super::toast_chrome::ToastPaint {
                modal_background: self.theme.modal_background,
                text: self.theme.text_primary,
                success: self.theme.success,
                info: self.theme.info,
                warning: self.theme.warning,
                error: self.theme.error,
            },
            0,
        );
    }
}
