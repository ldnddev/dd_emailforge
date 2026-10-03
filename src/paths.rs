use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub fn config_dir() -> PathBuf {
    config_dir_from(
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    )
}

#[allow(dead_code)]
pub fn config_dir_from(xdg: Option<OsString>, home: Option<OsString>) -> PathBuf {
    match xdg {
        Some(xdg) if !xdg.is_empty() => PathBuf::from(xdg).join("ldnddev"),
        _ => PathBuf::from(home.unwrap_or_default())
            .join(".config")
            .join("ldnddev"),
    }
}

#[allow(dead_code)]
pub fn theme_global() -> PathBuf {
    config_dir().join("dd_emailforge_theme.yml")
}

#[allow(dead_code)]
pub fn library_dir() -> PathBuf {
    config_dir().join("dd_emailforge").join("templates")
}

/// One-line file holding the last opened `template.json` path.
pub fn last_template_file() -> PathBuf {
    last_template_file_from(&config_dir())
}

pub fn last_template_file_from(config: &Path) -> PathBuf {
    config.join("dd_emailforge").join("last_path")
}

/// Ordered (path, source-label) pairs. First existing valid `version: 1` file wins.
pub fn theme_candidates() -> Vec<(PathBuf, &'static str)> {
    theme_candidates_from(
        PathBuf::from("dd_emailforge_theme.yml"),
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    )
}

pub fn theme_candidates_from(
    local: PathBuf,
    xdg: Option<OsString>,
    home: Option<OsString>,
) -> Vec<(PathBuf, &'static str)> {
    let mut c = vec![(local, "local")];
    let global = match xdg {
        Some(xdg) if !xdg.is_empty() => PathBuf::from(xdg)
            .join("ldnddev")
            .join("dd_emailforge_theme.yml"),
        _ => match home {
            Some(home) => PathBuf::from(home)
                .join(".config")
                .join("ldnddev")
                .join("dd_emailforge_theme.yml"),
            None => return c,
        },
    };
    c.push((global, "global"));
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn theme_candidates_honors_xdg_when_set() {
        let c = theme_candidates_from(
            PathBuf::from("dd_emailforge_theme.yml"),
            Some(OsString::from("/tmp/xdg-config")),
            Some(OsString::from("/home/user")),
        );
        assert_eq!(c.len(), 2);
        assert_eq!(c[0], (PathBuf::from("dd_emailforge_theme.yml"), "local"));
        assert_eq!(
            c[1],
            (
                PathBuf::from("/tmp/xdg-config/ldnddev/dd_emailforge_theme.yml"),
                "global"
            )
        );
    }

    #[test]
    fn theme_candidates_uses_home_when_xdg_unset() {
        let c = theme_candidates_from(
            PathBuf::from("dd_emailforge_theme.yml"),
            None,
            Some(OsString::from("/home/user")),
        );
        assert_eq!(
            c[1],
            (
                PathBuf::from("/home/user/.config/ldnddev/dd_emailforge_theme.yml"),
                "global"
            )
        );
    }

    #[test]
    fn theme_candidates_ignores_empty_xdg() {
        let c = theme_candidates_from(
            PathBuf::from("dd_emailforge_theme.yml"),
            Some(OsString::from("")),
            Some(OsString::from("/home/user")),
        );
        assert_eq!(
            c[1].0,
            PathBuf::from("/home/user/.config/ldnddev/dd_emailforge_theme.yml")
        );
    }

    #[test]
    fn config_dir_honors_xdg() {
        let dir = config_dir_from(
            Some(OsString::from("/tmp/xdg-config")),
            Some(OsString::from("/home/user")),
        );
        assert_eq!(dir, PathBuf::from("/tmp/xdg-config/ldnddev"));
    }

    #[test]
    fn config_helpers_point_at_ldnddev() {
        assert!(config_dir().ends_with("ldnddev"));
        assert!(theme_global().ends_with("dd_emailforge_theme.yml"));
        assert!(library_dir().ends_with("dd_emailforge/templates"));
        assert!(last_template_file().ends_with("dd_emailforge/last_path"));
        assert_eq!(
            last_template_file_from(&PathBuf::from("/tmp/xdg-config/ldnddev")),
            PathBuf::from("/tmp/xdg-config/ldnddev/dd_emailforge/last_path")
        );
    }
}
