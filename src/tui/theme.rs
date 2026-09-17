use std::path::PathBuf;

use ratatui::style::{Color, Style};
use serde::Deserialize;

use crate::paths::theme_candidates;

#[derive(Clone)]
pub(crate) struct AppTheme {
    pub(crate) base_background: Color,
    pub(crate) body_background: Color,
    pub(crate) modal_background: Color,
    pub(crate) text_primary: Color,
    pub(crate) text_secondary: Color,
    pub(crate) text_disabled: Color,
    pub(crate) text_inverse: Color,
    pub(crate) text_labels: Color,
    pub(crate) text_active_focus: Color,
    pub(crate) modal_labels: Color,
    pub(crate) modal_text: Color,
    pub(crate) modal_header: Color,
    pub(crate) selected_background: Color,
    pub(crate) border_default: Color,
    pub(crate) border_active: Color,
    pub(crate) scrollbar: Color,
    pub(crate) scrollbar_hover: Color,
    pub(crate) input_border_default: Color,
    pub(crate) input_border_focus: Color,
    pub(crate) input_text_default: Color,
    pub(crate) input_text_focus: Color,
    pub(crate) cursor: Color,
    pub(crate) success: Color,
    pub(crate) warning: Color,
    pub(crate) error: Color,
    pub(crate) info: Color,
    pub(crate) folders: Color,
    pub(crate) files: Color,
    pub(crate) links: Color,
    pub(crate) app_shell: Style,
    pub(crate) active_border: Style,
    pub(crate) header_quotes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ThemeFile {
    #[serde(default)]
    version: Option<u32>,
    #[serde(default)]
    header_quotes: Vec<String>,
    colors: PaletteFile,
}

#[derive(Debug, Deserialize)]
struct PaletteFile {
    base_background: String,
    body_background: String,
    modal_background: String,
    text_primary: String,
    text_secondary: String,
    text_disabled: Option<String>,
    text_inverse: Option<String>,
    text_labels: String,
    text_active_focus: String,
    modal_labels: String,
    modal_text: String,
    modal_header: String,
    selected_background: String,
    border_default: String,
    border_active: String,
    scrollbar: String,
    scrollbar_hover: String,
    input_border_default: String,
    input_border_focus: String,
    input_text_default: String,
    input_text_focus: String,
    cursor: String,
    success: String,
    warning: String,
    error: String,
    info: String,
    folders: String,
    files: String,
    links: String,
}

impl AppTheme {
    pub(crate) fn load() -> (Self, String, Option<String>) {
        Self::load_from(theme_candidates())
    }

    pub(crate) fn load_from(
        candidates: Vec<(PathBuf, &'static str)>,
    ) -> (Self, String, Option<String>) {
        let mut warning: Option<String> = None;

        for (path, src) in candidates {
            if !path.exists() {
                continue;
            }
            let raw = match std::fs::read_to_string(&path) {
                Ok(r) => r,
                Err(e) => {
                    warning = Some(format!("could not read '{}': {}", path.display(), e));
                    continue;
                }
            };
            let theme_file: ThemeFile = match serde_yaml::from_str(&raw) {
                Ok(f) => f,
                Err(e) => {
                    warning = Some(format!("invalid theme file '{}': {}", path.display(), e));
                    continue;
                }
            };

            match theme_file.version {
                Some(1) => {}
                Some(v) => {
                    warning = Some(format!(
                        "theme '{}' declares version {} (expected 1); using built-in defaults",
                        path.display(),
                        v
                    ));
                    continue;
                }
                None => {
                    warning = Some(format!(
                        "theme '{}' is missing required 'version: 1'; using built-in defaults",
                        path.display()
                    ));
                    continue;
                }
            }

            let quotes = if !theme_file.header_quotes.is_empty() {
                theme_file.header_quotes
            } else {
                default_header_quotes()
            };

            match Self::from_palette(theme_file.colors, quotes) {
                Ok(t) => return (t, src.to_string(), warning),
                Err(e) => {
                    warning = Some(format!(
                        "theme '{}' color parse error: {}; using defaults",
                        path.display(),
                        e
                    ));
                    continue;
                }
            }
        }

        (Self::default(), "default".to_string(), warning)
    }

    fn from_palette(p: PaletteFile, header_quotes: Vec<String>) -> anyhow::Result<Self> {
        let base_background = parse_hex_color(&p.base_background)?;
        let body_background = parse_hex_color(&p.body_background)?;
        let modal_background = parse_hex_color(&p.modal_background)?;
        let text_primary = parse_hex_color(&p.text_primary)?;
        let text_secondary = parse_hex_color(&p.text_secondary)?;
        let text_disabled = parse_hex_color(p.text_disabled.as_deref().unwrap_or("#A0A4A8"))?;
        let text_inverse = parse_hex_color(p.text_inverse.as_deref().unwrap_or("#F9FAFB"))?;
        let text_labels = parse_hex_color(&p.text_labels)?;
        let text_active_focus = parse_hex_color(&p.text_active_focus)?;
        let modal_labels = parse_hex_color(&p.modal_labels)?;
        let modal_text = parse_hex_color(&p.modal_text)?;
        let modal_header = parse_hex_color(&p.modal_header)?;
        let selected_background = parse_hex_color(&p.selected_background)?;
        let border_default = parse_hex_color(&p.border_default)?;
        let border_active = parse_hex_color(&p.border_active)?;
        let scrollbar = parse_hex_color(&p.scrollbar)?;
        let scrollbar_hover = parse_hex_color(&p.scrollbar_hover)?;
        let input_border_default = parse_hex_color(&p.input_border_default)?;
        let input_border_focus = parse_hex_color(&p.input_border_focus)?;
        let input_text_default = parse_hex_color(&p.input_text_default)?;
        let input_text_focus = parse_hex_color(&p.input_text_focus)?;
        let cursor = parse_hex_color(&p.cursor)?;
        let success = parse_hex_color(&p.success)?;
        let warning = parse_hex_color(&p.warning)?;
        let error = parse_hex_color(&p.error)?;
        let info = parse_hex_color(&p.info)?;
        let folders = parse_hex_color(&p.folders)?;
        let files = parse_hex_color(&p.files)?;
        let links = parse_hex_color(&p.links)?;

        let app_shell = Style::default().bg(base_background).fg(text_primary);
        let active_border = Style::default().fg(border_active);

        Ok(Self {
            base_background,
            body_background,
            modal_background,
            text_primary,
            text_secondary,
            text_disabled,
            text_inverse,
            text_labels,
            text_active_focus,
            modal_labels,
            modal_text,
            modal_header,
            selected_background,
            border_default,
            border_active,
            scrollbar,
            scrollbar_hover,
            input_border_default,
            input_border_focus,
            input_text_default,
            input_text_focus,
            cursor,
            success,
            warning,
            error,
            info,
            folders,
            files,
            links,
            app_shell,
            active_border,
            header_quotes,
        })
    }
}

impl Default for AppTheme {
    fn default() -> Self {
        let border_focus = Color::Rgb(100, 180, 245);
        let base = Color::Rgb(15, 17, 20);
        let text = Color::Rgb(245, 246, 247);
        Self {
            base_background: base,
            body_background: Color::Rgb(42, 45, 49),
            modal_background: Color::Rgb(28, 30, 33),
            text_primary: text,
            text_secondary: Color::Rgb(158, 163, 170),
            text_disabled: Color::Rgb(160, 164, 168),
            text_inverse: Color::Rgb(249, 250, 251),
            text_labels: Color::Rgb(255, 175, 70),
            text_active_focus: border_focus,
            modal_labels: border_focus,
            modal_text: text,
            modal_header: border_focus,
            selected_background: base,
            border_default: text,
            border_active: border_focus,
            scrollbar: Color::Rgb(255, 160, 135),
            scrollbar_hover: border_focus,
            input_border_default: text,
            input_border_focus: border_focus,
            input_text_default: text,
            input_text_focus: border_focus,
            cursor: border_focus,
            success: Color::Rgb(130, 224, 170),
            warning: Color::Rgb(245, 196, 105),
            error: Color::Rgb(229, 115, 115),
            info: Color::Rgb(93, 173, 226),
            folders: border_focus,
            files: Color::Rgb(255, 175, 70),
            links: Color::Rgb(255, 160, 135),
            app_shell: Style::default().bg(base).fg(text),
            active_border: Style::default().fg(border_focus),
            header_quotes: default_header_quotes(),
        }
    }
}

pub(crate) fn default_header_quotes() -> Vec<String> {
    vec![
        "Subject lines are just clickbait with better manners.".to_string(),
        "Pixel-perfect until Outlook opens it.".to_string(),
        "Preheader: the hallway before the party.".to_string(),
        "600 pixels wide. Infinite opinions.".to_string(),
        "Merge tags in, hope out.".to_string(),
    ]
}

pub(crate) fn choose_header_copy(quotes: &[String]) -> String {
    if quotes.is_empty() {
        return "Subject lines are just clickbait with better manners.".to_string();
    }
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
        ^ (std::process::id() as u64);
    quotes[(seed as usize) % quotes.len()].clone()
}

pub(crate) fn try_parse_hex_color(raw: &str) -> Option<Color> {
    parse_hex_color(raw).ok()
}

fn parse_hex_color(raw: &str) -> anyhow::Result<Color> {
    let hex = raw.trim().trim_start_matches('#');
    let expanded = match hex.len() {
        3 if hex.chars().all(|c| c.is_ascii_hexdigit()) => {
            let b = hex.as_bytes();
            format!(
                "{0}{0}{1}{1}{2}{2}",
                b[0] as char, b[1] as char, b[2] as char
            )
        }
        6 if hex.chars().all(|c| c.is_ascii_hexdigit()) => hex.to_string(),
        _ => {
            return Err(anyhow::anyhow!(
                "expected hex color like '#RRGGBB', got '{}'",
                raw
            ));
        }
    };
    let r = u8::from_str_radix(&expanded[0..2], 16)?;
    let g = u8::from_str_radix(&expanded[2..4], 16)?;
    let b = u8::from_str_radix(&expanded[4..6], 16)?;
    Ok(Color::Rgb(r, g, b))
}

pub(crate) fn color_to_hex(c: Color) -> String {
    if let Color::Rgb(r, g, b) = c {
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    } else {
        "?".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_yaml(contents: &str) -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "dd_emailforge_theme_{}_{}.yml",
            std::process::id(),
            n
        ));
        fs::write(&path, contents).expect("write temp theme");
        path
    }

    fn valid_yaml(version_line: &str) -> String {
        format!(
            r##"{version_line}
colors:
  base_background: "#0F1114"
  body_background: "#2A2D31"
  modal_background: "#1C1E21"
  text_primary: "#F5F6F7"
  text_secondary: "#9EA3AA"
  text_labels: "#FFAF46"
  text_active_focus: "#64B4F5"
  modal_labels: "#64B4F5"
  modal_text: "#F5F6F7"
  modal_header: "#64B4F5"
  selected_background: "#0F1114"
  border_default: "#F5F6F7"
  border_active: "#64B4F5"
  scrollbar: "#FFA087"
  scrollbar_hover: "#64B4F5"
  input_border_default: "#F5F6F7"
  input_border_focus: "#64B4F5"
  input_text_default: "#F5F6F7"
  input_text_focus: "#64B4F5"
  cursor: "#64B4F5"
  success: "#82e0aa"
  warning: "#f5c469"
  error: "#e57373"
  info: "#5dade2"
  folders: "#64B4F5"
  files: "#FFAF46"
  links: "#FFA087"
"##
        )
    }

    #[test]
    fn load_falls_back_to_default_when_no_files() {
        let (theme, source, warning) = AppTheme::load_from(vec![]);
        assert_eq!(source, "default");
        assert!(warning.is_none());
        assert_eq!(theme.base_background, Color::Rgb(15, 17, 20));
    }

    #[test]
    fn load_accepts_version_1() {
        let path = temp_yaml(&valid_yaml("version: 1"));
        let (theme, source, warning) = AppTheme::load_from(vec![(path.clone(), "local")]);
        let _ = fs::remove_file(&path);
        assert_eq!(source, "local");
        assert!(warning.is_none());
        assert_eq!(theme.base_background, Color::Rgb(15, 17, 20));
        assert_eq!(theme.body_background, Color::Rgb(42, 45, 49));
    }

    #[test]
    fn load_skips_version_2_with_warning() {
        let path = temp_yaml(&valid_yaml("version: 2"));
        let (_theme, source, warning) = AppTheme::load_from(vec![(path.clone(), "local")]);
        let _ = fs::remove_file(&path);
        assert_eq!(source, "default");
        let msg = warning.expect("warning");
        assert!(msg.contains("expected 1"), "{msg}");
    }

    #[test]
    fn load_skips_missing_version_with_warning() {
        let path = temp_yaml(&valid_yaml(""));
        let (_theme, source, warning) = AppTheme::load_from(vec![(path.clone(), "local")]);
        let _ = fs::remove_file(&path);
        assert_eq!(source, "default");
        let msg = warning.expect("warning");
        assert!(msg.contains("version: 1"), "{msg}");
    }

    #[test]
    fn choose_header_copy_picks_a_builtin() {
        let quotes = default_header_quotes();
        let picked = choose_header_copy(&quotes);
        assert!(quotes.contains(&picked), "{picked}");
    }

    #[test]
    fn load_uses_header_quotes_override() {
        let mut yaml = valid_yaml("version: 1");
        yaml.push_str("header_quotes:\n  - Custom tagline only.\n");
        let path = temp_yaml(&yaml);
        let (theme, _, _) = AppTheme::load_from(vec![(path.clone(), "local")]);
        let _ = fs::remove_file(&path);
        assert_eq!(
            theme.header_quotes,
            vec!["Custom tagline only.".to_string()]
        );
    }
}
