use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::model::Template;

#[derive(Debug)]
pub enum LoadError {
    MissingVersion,
    UnsupportedVersion(u32),
    Parse(String),
    Io(io::Error),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::MissingVersion => {
                write!(f, "missing template.json version (expected 1)")
            }
            LoadError::UnsupportedVersion(n) => {
                write!(f, "unsupported template.json version {n} (expected 1)")
            }
            LoadError::Parse(msg) => write!(f, "failed to parse template.json: {msg}"),
            LoadError::Io(err) => write!(f, "failed to read template.json: {err}"),
        }
    }
}

impl std::error::Error for LoadError {}

#[derive(Deserialize)]
struct VersionPeek {
    version: Option<u32>,
}

pub fn resolve_template_path(arg: &Path) -> anyhow::Result<PathBuf> {
    if arg.is_dir() {
        let json = arg.join("template.json");
        if json.is_file() {
            return Ok(json);
        }
        anyhow::bail!("no template.json in directory '{}'", arg.display());
    }
    if arg.is_file() {
        return Ok(arg.to_path_buf());
    }
    anyhow::bail!("template path not found: '{}'", arg.display());
}

pub fn template_root(json_path: &Path) -> PathBuf {
    json_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn backup_path_for(path: &Path) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(".backup");
    PathBuf::from(s)
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    let tmp = {
        let mut name = path
            .file_name()
            .map(|n| n.to_os_string())
            .unwrap_or_else(|| "template.json".into());
        name.push(".tmp");
        match path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.join(name),
            _ => Path::new(".").join(name),
        }
    };
    fs::write(&tmp, bytes)
        .map_err(|e| anyhow::anyhow!("failed to write temp file '{}': {e}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|e| {
        anyhow::anyhow!(
            "failed to rename '{}' -> '{}': {e}",
            tmp.display(),
            path.display()
        )
    })?;
    Ok(())
}

pub fn save_template(path: &Path, template: &Template) -> anyhow::Result<()> {
    let mut t = template.clone();
    t.normalize_base_url();
    let mut json = serde_json::to_string_pretty(&t)
        .map_err(|e| anyhow::anyhow!("failed to serialize template: {e}"))?;
    if !json.ends_with('\n') {
        json.push('\n');
    }
    atomic_write(path, json.as_bytes())?;
    Ok(())
}

/// Persist `json_path` so a bare `dd_emailforge` can reopen it.
/// No-op under `cargo test` so TUI tests do not write into `~/.config`.
pub fn remember_last_template(json_path: &Path) {
    if cfg!(test) {
        return;
    }
    let _ = remember_last_template_at(&crate::paths::config_dir(), json_path);
}

pub fn remember_last_template_at(config: &Path, json_path: &Path) -> io::Result<()> {
    let file = crate::paths::last_template_file_from(config);
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }
    let stored = json_path
        .canonicalize()
        .unwrap_or_else(|_| json_path.to_path_buf());
    fs::write(file, stored.to_string_lossy().as_bytes())
}

/// Last remembered `template.json`, if that path still resolves.
pub fn load_last_template() -> Option<PathBuf> {
    if cfg!(test) {
        return None;
    }
    load_last_template_at(&crate::paths::config_dir())
}

pub fn load_last_template_at(config: &Path) -> Option<PathBuf> {
    let file = crate::paths::last_template_file_from(config);
    let raw = fs::read_to_string(&file).ok()?;
    let p = PathBuf::from(raw.trim());
    if p.as_os_str().is_empty() {
        return None;
    }
    match resolve_template_path(&p) {
        Ok(json) => Some(json),
        Err(_) => {
            let _ = fs::remove_file(file);
            None
        }
    }
}

pub fn load_template(path: &Path) -> Result<Template, LoadError> {
    let raw = fs::read_to_string(path).map_err(LoadError::Io)?;
    let peek: VersionPeek =
        serde_json::from_str(&raw).map_err(|e| LoadError::Parse(e.to_string()))?;
    match peek.version {
        None => return Err(LoadError::MissingVersion),
        Some(1) => {}
        Some(n) => return Err(LoadError::UnsupportedVersion(n)),
    }
    serde_json::from_str(&raw).map_err(|e| LoadError::Parse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Template;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "dd_emailforge_storage_{}_{}",
            std::process::id(),
            n
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolve_directory_finds_template_json() {
        let dir = temp_dir();
        let json = dir.join("template.json");
        fs::write(&json, "{}").unwrap();
        assert_eq!(resolve_template_path(&dir).unwrap(), json);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_missing_path_errors() {
        let err = resolve_template_path(Path::new("/no/such/template.json")).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn load_missing_version() {
        let dir = temp_dir();
        let json = dir.join("template.json");
        fs::write(
            &json,
            r#"{"name":"n","subject":"s","brand":{},"head":{"title":"t"},"body":{}}"#,
        )
        .unwrap();
        match load_template(&json) {
            Err(LoadError::MissingVersion) => {}
            other => panic!("expected MissingVersion, got {other:?}"),
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_unsupported_version() {
        let dir = temp_dir();
        let json = dir.join("template.json");
        fs::write(
            &json,
            r#"{"version":2,"name":"n","subject":"s","brand":{},"head":{"title":"t"},"body":{}}"#,
        )
        .unwrap();
        match load_template(&json) {
            Err(LoadError::UnsupportedVersion(2)) => {}
            other => panic!("expected UnsupportedVersion(2), got {other:?}"),
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_round_trip_and_no_tmp_left() {
        let dir = temp_dir();
        let json = dir.join("template.json");
        let mut t = Template::minimal();
        t.base_url = "https://cdn.example.com".to_string();
        save_template(&json, &t).unwrap();
        assert!(!dir.join("template.json.tmp").exists());
        let loaded = load_template(&json).unwrap();
        assert_eq!(loaded.base_url, "https://cdn.example.com/");
        assert_eq!(loaded.name, "welcome");
        let raw = fs::read_to_string(&json).unwrap();
        assert!(raw.ends_with('\n'));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn backup_path_appends_backup() {
        let p = PathBuf::from("/tmp/template.json");
        assert_eq!(
            backup_path_for(&p),
            PathBuf::from("/tmp/template.json.backup")
        );
    }

    #[test]
    fn show_normalizes_unknown_keys() {
        let dir = temp_dir();
        let json = dir.join("template.json");
        fs::write(
            &json,
            r#"{
                "version": 1,
                "name": "n",
                "subject": "s",
                "mystery": true,
                "brand": {},
                "head": { "title": "t" },
                "body": { "nodes": [] }
            }"#,
        )
        .unwrap();
        let t = load_template(&json).unwrap();
        let dumped = serde_json::to_string(&t).unwrap();
        assert!(!dumped.contains("mystery"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn remember_and_load_last_template_round_trip() {
        let config = temp_dir();
        let tmpl = temp_dir();
        let json = tmpl.join("template.json");
        fs::write(&json, "{}").unwrap();
        remember_last_template_at(&config, &json).unwrap();
        let loaded = load_last_template_at(&config).expect("last path");
        assert_eq!(loaded, json.canonicalize().unwrap());
        let _ = fs::remove_dir_all(&config);
        let _ = fs::remove_dir_all(&tmpl);
    }

    #[test]
    fn load_last_template_forgets_missing_path() {
        let config = temp_dir();
        let missing = PathBuf::from("/no/such/dd_emailforge/template.json");
        remember_last_template_at(&config, &missing).unwrap();
        assert!(load_last_template_at(&config).is_none());
        assert!(!crate::paths::last_template_file_from(&config).exists());
        let _ = fs::remove_dir_all(&config);
    }

    #[test]
    fn load_last_template_accepts_directory_argument() {
        let config = temp_dir();
        let tmpl = temp_dir();
        let json = tmpl.join("template.json");
        fs::write(&json, "{}").unwrap();
        remember_last_template_at(&config, &tmpl).unwrap();
        let loaded = load_last_template_at(&config).expect("dir should resolve");
        assert_eq!(loaded, json.canonicalize().unwrap());
        let _ = fs::remove_dir_all(&config);
        let _ = fs::remove_dir_all(&tmpl);
    }
}
