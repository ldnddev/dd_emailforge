//! Shortcut sheet. The first row is the shared Settings chord.

pub struct Shortcut {
    pub section: &'static str,
    pub action: &'static str,
    pub gui: &'static str,
    pub tui: &'static str,
}

pub fn rows() -> &'static [Shortcut] {
    &[
        Shortcut {
            section: "Global",
            action: "Settings",
            gui: "F1",
            tui: "F1",
        },
        Shortcut {
            section: "Global",
            action: "Validate",
            gui: "F3",
            tui: "F3",
        },
        Shortcut {
            section: "Global",
            action: "Quit",
            gui: "Ctrl+Q",
            tui: "Ctrl+Q",
        },
        Shortcut {
            section: "Global",
            action: "New template",
            gui: "N",
            tui: "N",
        },
        Shortcut {
            section: "Global",
            action: "Open template",
            gui: "O",
            tui: "O",
        },
        Shortcut {
            section: "Global",
            action: "Save template",
            gui: "Ctrl+S",
            tui: "S",
        },
        Shortcut {
            section: "Settings",
            action: "Close settings",
            gui: "Esc",
            tui: "Esc",
        },
        Shortcut {
            section: "Settings",
            action: "Save theme",
            gui: "Y",
            tui: "Y",
        },
        Shortcut {
            section: "Settings",
            action: "Toggle save target",
            gui: "T",
            tui: "T",
        },
    ]
}

/// Help page lines. The first row is `Settings · F1 · F1`.
pub fn lines() -> Vec<String> {
    let mut out = Vec::new();
    let mut section = "";
    for (index, row) in rows().iter().enumerate() {
        if index == 0 {
            out.push(format!("{} · {} · {}", row.action, row.gui, row.tui));
        }
        if row.section != section {
            section = row.section;
            out.push(section.to_string());
        }
        if index != 0 {
            out.push(format!("{} · {} · {}", row.action, row.gui, row.tui));
        }
    }
    out
}
