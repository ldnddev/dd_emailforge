//! Window preferences. They live in `gui.json`, never in `template.json`
//! and never in the theme YAML.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Density {
    #[default]
    Comfortable,
    Compact,
}

impl Density {
    pub fn label(self) -> &'static str {
        match self {
            Self::Comfortable => "Comfortable",
            Self::Compact => "Compact",
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            Self::Comfortable => Self::Compact,
            Self::Compact => Self::Comfortable,
        }
    }

    pub fn row_px(self) -> f32 {
        match self {
            Self::Comfortable => 28.0,
            Self::Compact => 24.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuiSettings {
    #[serde(default)]
    pub density: Density,
    #[serde(default = "default_true")]
    pub vim_keys: bool,
    #[serde(default)]
    pub reduced_motion: bool,
}

fn default_true() -> bool {
    true
}

impl Default for GuiSettings {
    fn default() -> Self {
        Self {
            density: Density::Comfortable,
            vim_keys: true,
            reduced_motion: false,
        }
    }
}

/// `$XDG_CONFIG_HOME/ldnddev/dd_emailforge/gui.json`.
pub fn settings_path() -> PathBuf {
    dd_emailforge::paths::config_dir()
        .join("dd_emailforge")
        .join("gui.json")
}

pub fn load_from(path: &Path) -> GuiSettings {
    let Ok(bytes) = fs::read(path) else {
        return GuiSettings::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

pub fn save_to(path: &Path, settings: &GuiSettings) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut json = serde_json::to_vec_pretty(settings).unwrap_or_else(|_| b"{}".to_vec());
    if !json.ends_with(b"\n") {
        json.push(b'\n');
    }
    fs::write(path, json)
}
