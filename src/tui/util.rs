/// Join `leading` then `trailing` with `sep`, dropping `leading` from the end
/// until the line fits `width`. If the tail still overflows, drop it from the
/// front so the last segment (usually cancel) stays.
pub(super) fn fit_segments(width: usize, leading: &[&str], trailing: &[&str], sep: &str) -> String {
    if width == 0 {
        return String::new();
    }
    let mut lead = leading.len();
    let mut trail_start = 0;
    loop {
        let mut parts: Vec<&str> =
            Vec::with_capacity(lead + trailing.len().saturating_sub(trail_start));
        parts.extend_from_slice(&leading[..lead]);
        parts.extend_from_slice(&trailing[trail_start..]);
        if parts.is_empty() {
            return String::new();
        }
        let line = parts.join(sep);
        if line.chars().count() <= width {
            return line;
        }
        if lead > 0 {
            lead -= 1;
        } else if trail_start + 1 < trailing.len() {
            trail_start += 1;
        } else {
            return line;
        }
    }
}

/// Spawn the OS-default opener on the given URL. All three stdio streams
/// are redirected to /dev/null so the TUI in raw mode is not scrambled.
pub(super) fn open_in_browser(target: &str) -> std::io::Result<()> {
    use std::process::{Command, Stdio};
    let mut cmd: Command;
    #[cfg(target_os = "linux")]
    {
        cmd = Command::new("xdg-open");
        cmd.arg(target);
    }
    #[cfg(target_os = "macos")]
    {
        cmd = Command::new("open");
        cmd.arg(target);
    }
    #[cfg(target_os = "windows")]
    {
        cmd = Command::new("cmd");
        cmd.args(["/C", "start", ""]).arg(target);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = target;
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "no known browser opener for this target",
        ));
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}

#[derive(Debug, Clone)]
pub(super) struct DirEntryRow {
    pub(super) name: String,
    pub(super) is_dir: bool,
}

/// List immediate children of `dir`, sorted: subdirs first (alpha), then
/// files (alpha). Hidden entries (leading dot) are skipped.
pub(super) fn list_dir_entries(dir: &std::path::Path) -> Vec<DirEntryRow> {
    let read = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let row = DirEntryRow { name, is_dir };
        if is_dir {
            dirs.push(row);
        } else {
            files.push(row);
        }
    }
    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    dirs.extend(files);
    dirs
}

pub(super) fn filter_entries(entries: &[DirEntryRow], filter: &str) -> Vec<DirEntryRow> {
    if filter.is_empty() {
        return entries.to_vec();
    }
    let needle = filter.to_lowercase();
    entries
        .iter()
        .filter(|e| e.name.to_lowercase().contains(&needle))
        .cloned()
        .collect()
}
