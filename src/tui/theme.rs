use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ldnddev_theme::{ParseMode, ThemeSource};
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
    pub(crate) omarchy: Option<ldnddev_theme::OmarchyChoice>,
    pub(crate) overrides: BTreeSet<String>,
}

#[derive(Debug, Deserialize)]
struct VersionProbe {
    #[serde(default)]
    version: Option<serde_yaml::Value>,
}

impl AppTheme {
    pub(crate) fn load() -> (Self, String, Option<String>) {
        Self::load_from(theme_candidates())
    }

    pub(crate) fn load_from(
        candidates: Vec<(PathBuf, &'static str)>,
    ) -> (Self, String, Option<String>) {
        Self::load_resolved(candidates, &ldnddev_theme::OmarchyPaths::system())
    }

    pub(crate) fn load_resolved(
        candidates: Vec<(PathBuf, &'static str)>,
        omarchy: &ldnddev_theme::OmarchyPaths,
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
            if let Err(msg) = accept_version(&raw, &path) {
                warning = Some(msg);
                continue;
            }
            match ldnddev_theme::load_from_str(&raw, ParseMode::Strict, theme_source(src)) {
                Ok(palette) => return finish_palette(palette, src, omarchy, warning),
                Err(e) => {
                    warning = Some(format!(
                        "theme '{}' color parse error: {}; using defaults",
                        path.display(),
                        e
                    ));
                }
            }
        }

        (Self::default(), "default".to_string(), warning)
    }

    fn from_palette(mut palette: ldnddev_theme::Palette) -> Self {
        palette.ensure_extras(extra_theme_fields());
        if palette.header_quotes.is_empty() {
            palette.header_quotes = default_header_quotes();
        }
        let get =
            |key: &str, fallback: Color| palette.get(key).map(rgb_to_color).unwrap_or(fallback);
        let base_background = get("base_background", Color::Rgb(15, 17, 20));
        let text_primary = get("text_primary", Color::Rgb(245, 246, 247));
        let border_active = get("border_active", Color::Rgb(100, 180, 245));
        Self {
            base_background,
            body_background: get("body_background", Color::Rgb(42, 45, 49)),
            modal_background: get("modal_background", Color::Rgb(28, 30, 33)),
            text_primary,
            text_secondary: get("text_secondary", Color::Rgb(158, 163, 170)),
            text_disabled: get("text_disabled", Color::Rgb(160, 164, 168)),
            text_inverse: get("text_inverse", Color::Rgb(249, 250, 251)),
            text_labels: get("text_labels", Color::Rgb(255, 175, 70)),
            text_active_focus: get("text_active_focus", border_active),
            modal_labels: get("modal_labels", border_active),
            modal_text: get("modal_text", text_primary),
            modal_header: get("modal_header", border_active),
            selected_background: get("selected_background", base_background),
            border_default: get("border_default", text_primary),
            border_active,
            scrollbar: get("scrollbar", Color::Rgb(255, 160, 135)),
            scrollbar_hover: get("scrollbar_hover", border_active),
            input_border_default: get("input_border_default", text_primary),
            input_border_focus: get("input_border_focus", border_active),
            input_text_default: get("input_text_default", text_primary),
            input_text_focus: get("input_text_focus", border_active),
            cursor: get("cursor", border_active),
            success: get("success", Color::Rgb(130, 224, 170)),
            warning: get("warning", Color::Rgb(245, 196, 105)),
            error: get("error", Color::Rgb(229, 115, 115)),
            info: get("info", Color::Rgb(93, 173, 226)),
            folders: get("folders", border_active),
            files: get("files", Color::Rgb(255, 175, 70)),
            links: get("links", Color::Rgb(255, 160, 135)),
            app_shell: Style::default().bg(base_background).fg(text_primary),
            active_border: Style::default().fg(border_active),
            header_quotes: palette.header_quotes,
            omarchy: palette.omarchy,
            overrides: palette.overrides,
        }
    }
}

fn finish_palette(
    palette: ldnddev_theme::Palette,
    source: &str,
    omarchy: &ldnddev_theme::OmarchyPaths,
    warning: Option<String>,
) -> (AppTheme, String, Option<String>) {
    match ldnddev_theme::apply_omarchy(&palette, omarchy) {
        Ok(resolved) => (
            AppTheme::from_palette(resolved),
            source.to_string(),
            warning,
        ),
        Err(err) => (
            AppTheme::default(),
            "default".to_string(),
            Some(format!(
                "omarchy theme failed ({err}); using built-in defaults"
            )),
        ),
    }
}

fn theme_source(src: &str) -> ThemeSource {
    match src {
        "global" => ThemeSource::Global,
        "default" => ThemeSource::Default,
        _ => ThemeSource::Local,
    }
}

/// Keep the historical version warnings. The shared loader's wording differs.
fn accept_version(raw: &str, path: &Path) -> Result<(), String> {
    let probe: VersionProbe = serde_yaml::from_str(raw)
        .map_err(|e| format!("invalid theme file '{}': {}", path.display(), e))?;
    match probe.version {
        Some(serde_yaml::Value::Number(n)) => match n.as_u64() {
            Some(1) => Ok(()),
            Some(v) => Err(format!(
                "theme '{}' declares version {} (expected 1); using built-in defaults",
                path.display(),
                v
            )),
            None => Err(format!(
                "invalid theme file '{}': version is not an integer",
                path.display()
            )),
        },
        None => Err(format!(
            "theme '{}' is missing required 'version: 1'; using built-in defaults",
            path.display()
        )),
        Some(_) => Err(format!(
            "invalid theme file '{}': version is not an integer",
            path.display()
        )),
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
            omarchy: None,
            overrides: BTreeSet::new(),
        }
    }
}

pub(crate) fn extra_theme_fields() -> &'static [ldnddev_theme::ColorField] {
    &[
        ldnddev_theme::EXTRA_MODAL_HEADER,
        ldnddev_theme::EXTRA_TEXT_DISABLED,
        ldnddev_theme::EXTRA_TEXT_INVERSE,
    ]
}

fn rgb_to_color(rgb: ldnddev_theme::Rgb) -> Color {
    Color::Rgb(rgb.r, rgb.g, rgb.b)
}

fn color_to_rgb(color: Color) -> ldnddev_theme::Rgb {
    match color {
        Color::Rgb(r, g, b) => ldnddev_theme::Rgb { r, g, b },
        _ => ldnddev_theme::Rgb { r: 0, g: 0, b: 0 },
    }
}

pub(crate) fn palette_from_theme(theme: &AppTheme) -> ldnddev_theme::Palette {
    let mut palette = ldnddev_theme::Palette::builtin();
    palette.header_quotes = theme.header_quotes.clone();
    palette.omarchy = theme.omarchy.clone();
    palette.overrides = theme.overrides.clone();
    for (key, color) in [
        ("base_background", theme.base_background),
        ("body_background", theme.body_background),
        ("modal_background", theme.modal_background),
        ("text_primary", theme.text_primary),
        ("text_secondary", theme.text_secondary),
        ("text_disabled", theme.text_disabled),
        ("text_inverse", theme.text_inverse),
        ("text_labels", theme.text_labels),
        ("text_active_focus", theme.text_active_focus),
        ("modal_labels", theme.modal_labels),
        ("modal_text", theme.modal_text),
        ("modal_header", theme.modal_header),
        ("selected_background", theme.selected_background),
        ("border_default", theme.border_default),
        ("border_active", theme.border_active),
        ("scrollbar", theme.scrollbar),
        ("scrollbar_hover", theme.scrollbar_hover),
        ("input_border_default", theme.input_border_default),
        ("input_border_focus", theme.input_border_focus),
        ("input_text_default", theme.input_text_default),
        ("input_text_focus", theme.input_text_focus),
        ("cursor", theme.cursor),
        ("success", theme.success),
        ("warning", theme.warning),
        ("error", theme.error),
        ("info", theme.info),
        ("folders", theme.folders),
        ("files", theme.files),
        ("links", theme.links),
    ] {
        palette.set(key, color_to_rgb(color));
    }
    palette
}

pub(crate) fn apply_palette(theme: &mut AppTheme, palette: &ldnddev_theme::Palette) {
    *theme = AppTheme::from_palette(palette.clone());
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

    #[test]
    fn omarchy_link_maps_colors_and_keeps_override() {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "dd_emailforge_omarchy_{}_{}",
            std::process::id(),
            n
        ));
        let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("crates/ldnddev_theme/tests/fixtures/omarchy/tokyo-night/colors.toml");
        fs::create_dir_all(root.join("stock/tokyo-night")).expect("mkdir");
        fs::copy(&src, root.join("stock/tokyo-night/colors.toml")).expect("copy");
        let yaml = "version: 1\nomarchy: tokyo-night\ncolors:\n  text_labels: \"#ABCDEF\"\n";
        let path = temp_yaml(yaml);
        let (theme, source, warning) = AppTheme::load_resolved(
            vec![(path.clone(), "local")],
            &ldnddev_theme::OmarchyPaths::from_root(&root),
        );
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&root);
        assert!(warning.is_none(), "{warning:?}");
        assert_eq!(source, "local");
        assert_eq!(color_to_hex(theme.body_background), "#1A1B26");
        assert_eq!(color_to_hex(theme.text_labels), "#ABCDEF");
        assert_eq!(
            theme.omarchy,
            Some(ldnddev_theme::OmarchyChoice::Theme("tokyo-night".into()))
        );
        assert!(theme.overrides.contains("text_labels"));
    }

    #[test]
    fn missing_omarchy_theme_uses_builtin_with_warning() {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "dd_emailforge_omarchy_missing_{}_{}",
            std::process::id(),
            n
        ));
        fs::create_dir_all(&root).expect("mkdir");
        let path = temp_yaml("version: 1\nomarchy: missing-theme\n");
        let (theme, source, warning) = AppTheme::load_resolved(
            vec![(path.clone(), "local")],
            &ldnddev_theme::OmarchyPaths::from_root(&root),
        );
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&root);
        let msg = warning.expect("warning");
        assert!(msg.contains("omarchy theme failed"), "{msg}");
        assert_eq!(source, "default");
        assert_eq!(theme.base_background, Color::Rgb(15, 17, 20));
        assert!(theme.omarchy.is_none());
    }
}
