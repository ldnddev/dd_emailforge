//! Same theme lookup as the terminal: `./dd_emailforge_theme.yml`, then
//! `$XDG_CONFIG_HOME/ldnddev/dd_emailforge_theme.yml`.

use gpui::{Hsla, Rgba};
use ldnddev_theme::{
    ColorField, OmarchyPaths, Palette, ParseMode, ThemeSource, COLOR_FIELDS, EXTRA_MODAL_HEADER,
    EXTRA_TEXT_DISABLED, EXTRA_TEXT_INVERSE,
};

pub struct LoadedTheme {
    pub palette: Palette,
    pub label: String,
    pub warning: Option<String>,
    pub paths: OmarchyPaths,
}

pub fn load_theme() -> LoadedTheme {
    let paths = OmarchyPaths::system();
    let mut warning: Option<String> = None;
    for (path, source) in dd_emailforge::paths::theme_candidates() {
        if !path.exists() {
            continue;
        }
        let raw = match std::fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(err) => {
                warning = Some(format!("could not read '{}': {err}", path.display()));
                continue;
            }
        };
        match ldnddev_theme::load_from_str(&raw, ParseMode::Strict, theme_source(source)) {
            Ok(palette) => match ldnddev_theme::apply_omarchy(&palette, &paths) {
                Ok(resolved) => {
                    return LoadedTheme {
                        palette: with_extras(resolved),
                        label: source.to_string(),
                        warning,
                        paths,
                    };
                }
                Err(err) => {
                    return LoadedTheme {
                        palette: with_extras(Palette::builtin()),
                        label: "default".to_string(),
                        warning: Some(format!(
                            "omarchy theme failed ({err}); using built-in defaults"
                        )),
                        paths,
                    };
                }
            },
            Err(err) => {
                warning = Some(format!(
                    "theme '{}' color parse error: {err}; using defaults",
                    path.display()
                ));
            }
        }
    }
    LoadedTheme {
        palette: with_extras(Palette::builtin()),
        label: "default".to_string(),
        warning,
        paths,
    }
}

fn with_extras(mut palette: Palette) -> Palette {
    palette.ensure_extras(theme_extras());
    palette
}

pub fn theme_extras() -> &'static [ColorField] {
    &[
        EXTRA_MODAL_HEADER,
        EXTRA_TEXT_DISABLED,
        EXTRA_TEXT_INVERSE,
    ]
}

pub fn theme_fields() -> Vec<ColorField> {
    let mut fields = COLOR_FIELDS.to_vec();
    for extra in theme_extras() {
        if !fields.iter().any(|field| field.key == extra.key) {
            fields.push(*extra);
        }
    }
    fields
}

fn theme_source(source: &str) -> ThemeSource {
    match source {
        "global" => ThemeSource::Global,
        "default" => ThemeSource::Default,
        _ => ThemeSource::Local,
    }
}

pub fn hsla(color: ldnddev_theme::Rgb) -> Hsla {
    Rgba {
        r: f32::from(color.r) / 255.0,
        g: f32::from(color.g) / 255.0,
        b: f32::from(color.b) / 255.0,
        a: 1.0,
    }
    .into()
}

#[derive(Clone, Copy)]
pub struct Chrome {
    pub base: Hsla,
    pub panel: Hsla,
    pub raised: Hsla,
    pub text: Hsla,
    pub secondary: Hsla,
    pub border: Hsla,
    pub selected: Hsla,
    pub accent: Hsla,
}

pub fn chrome(palette: &Palette) -> Chrome {
    let color = |key: &str, fallback: ldnddev_theme::Rgb| {
        hsla(palette.get(key).unwrap_or(fallback))
    };
    Chrome {
        base: color("base_background", ldnddev_theme::Rgb::new(0x0f, 0x11, 0x14)),
        panel: color("modal_background", ldnddev_theme::Rgb::new(0x1c, 0x1e, 0x21)),
        raised: color("body_background", ldnddev_theme::Rgb::new(0x2a, 0x2d, 0x31)),
        text: color("text_primary", ldnddev_theme::Rgb::new(0xf5, 0xf6, 0xf7)),
        secondary: color("text_secondary", ldnddev_theme::Rgb::new(0x9e, 0xa3, 0xaa)),
        border: color("border_default", ldnddev_theme::Rgb::new(0xf5, 0xf6, 0xf7)),
        selected: color(
            "selected_background",
            ldnddev_theme::Rgb::new(0x20, 0x60, 0xa0),
        ),
        accent: color("border_active", ldnddev_theme::Rgb::new(0x64, 0xb4, 0xf5)),
    }
}

