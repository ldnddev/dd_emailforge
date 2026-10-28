//! Official MJML 5 CLI discovery, one-shot compile, and `mjml -w`.
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

pub const TEMPLATE_SYNTAX: &str = r#"[{"prefix":"{{","suffix":"}}"},{"prefix":"[[","suffix":"]]"},{"prefix":"*|","suffix":"|*"},{"prefix":"{%","suffix":"%}"},{"prefix":"%%","suffix":"%%"},{"prefix":"<%","suffix":"%>"}]"#;

const COMPILE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub enum MjmlError {
    NotFound { searched: Vec<String> },
    Compile { stderr: String },
    Timeout,
    Io(std::io::Error),
}

impl std::fmt::Display for MjmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MjmlError::NotFound { searched } => {
                writeln!(f, "mjml CLI not found. Install the pin from this template:")?;
                writeln!(f, "  cd <template-dir> && npm install")?;
                writeln!(f, "Looked in:")?;
                for s in searched {
                    writeln!(f, "  {s}")?;
                }
                Ok(())
            }
            MjmlError::Compile { stderr } => write!(f, "mjml compile failed:\n{stderr}"),
            MjmlError::Timeout => write!(f, "mjml compile timed out after 30s"),
            MjmlError::Io(e) => write!(f, "mjml io error: {e}"),
        }
    }
}

impl std::error::Error for MjmlError {}

pub fn not_found_message(searched: &[String]) -> String {
    let mut s = String::from(
        "mjml CLI not found. Install the pin from this template:\n  cd <template-dir> && npm install\nLooked in:\n",
    );
    for p in searched {
        s.push_str("  ");
        s.push_str(p);
        s.push('\n');
    }
    s
}

pub fn discover_mjml(template_root: &Path) -> Result<PathBuf, MjmlError> {
    let local = template_root.join("node_modules").join(".bin").join("mjml");
    let mut searched = vec![local.display().to_string()];
    if local.is_file() {
        return Ok(local);
    }
    if let Some(on_path) = which_mjml() {
        return Ok(on_path);
    }
    searched.push("$PATH".to_string());
    Err(MjmlError::NotFound { searched })
}

fn which_mjml() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join("mjml");
        if cand.is_file() {
            return Some(cand);
        }
        #[cfg(windows)]
        {
            let cmd = dir.join("mjml.cmd");
            if cmd.is_file() {
                return Some(cmd);
            }
        }
    }
    None
}

fn push_mjml_flags(cmd: &mut Command) {
    cmd.arg("--config.validationLevel")
        .arg("strict")
        .arg("--config.beautify")
        .arg("true")
        .arg("--config.keepComments")
        .arg("true")
        .arg("--config.sanitizeStyles")
        .arg("true")
        .arg("--config.allowMixedSyntax")
        .arg("true")
        .arg("--config.templateSyntax")
        .arg(TEMPLATE_SYNTAX);
}

pub struct CompileResult {
    pub html_bytes: u64,
}

pub fn compile_one_shot_captured(
    bin: &Path,
    cwd: &Path,
    input: &Path,
    output: &Path,
) -> Result<CompileResult, MjmlError> {
    let mut cmd = Command::new(bin);
    cmd.current_dir(cwd)
        .arg(input)
        .arg("-o")
        .arg(output)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    push_mjml_flags(&mut cmd);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut child = cmd.spawn().map_err(MjmlError::Io)?;
    let pid = child.id();
    let stderr_pipe = child.stderr.take();
    let stderr_handle = std::thread::spawn(move || {
        let mut buf = String::new();
        if let Some(pipe) = stderr_pipe {
            let _ = std::io::Read::read_to_string(&mut BufReader::new(pipe), &mut buf);
        }
        buf
    });
    match wait_with_timeout(&mut child, COMPILE_TIMEOUT).map_err(MjmlError::Io)? {
        None => {
            kill_group(pid);
            let _ = child.wait();
            let _ = stderr_handle.join();
            Err(MjmlError::Timeout)
        }
        Some(status) => {
            let stderr = stderr_handle.join().unwrap_or_default();
            if !status.success() {
                return Err(MjmlError::Compile { stderr });
            }
            let html_bytes = std::fs::metadata(output).map(|m| m.len()).unwrap_or(0);
            Ok(CompileResult { html_bytes })
        }
    }
}

fn wait_with_timeout(
    child: &mut Child,
    dur: Duration,
) -> std::io::Result<Option<std::process::ExitStatus>> {
    let start = Instant::now();
    loop {
        if let Some(st) = child.try_wait()? {
            return Ok(Some(st));
        }
        if start.elapsed() >= dur {
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn kill_group(pid: u32) {
    #[cfg(unix)]
    unsafe {
        unsafe extern "C" {
            fn kill(pid: i32, sig: i32) -> i32;
        }
        const SIGTERM: i32 = 15;
        const SIGKILL: i32 = 9;
        let pg = -(pid as i32);
        let _ = kill(pg, SIGTERM);
        std::thread::sleep(Duration::from_millis(500));
        let _ = kill(pg, SIGKILL);
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
    }
}

pub struct MjmlWatch {
    child: Child,
    pid: u32,
    pub errors: Receiver<String>,
}

impl MjmlWatch {
    pub fn spawn(bin: &Path, cwd: &Path, input: &Path, output: &Path) -> Result<Self, MjmlError> {
        let mut cmd = Command::new(bin);
        cmd.current_dir(cwd)
            .arg("-w")
            .arg(input)
            .arg("-o")
            .arg(output)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        push_mjml_flags(&mut cmd);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        let mut child = cmd.spawn().map_err(MjmlError::Io)?;
        let pid = child.id();
        let stderr = child.stderr.take();
        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        std::thread::spawn(move || {
            let Some(pipe) = stderr else { return };
            let reader = BufReader::new(pipe);
            for line in reader.lines().map_while(Result::ok) {
                if line_looks_like_error(&line) {
                    let _ = tx.send(line);
                }
            }
        });
        Ok(Self {
            child,
            pid,
            errors: rx,
        })
    }
}

impl Drop for MjmlWatch {
    fn drop(&mut self) {
        kill_group(self.pid);
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn line_looks_like_error(line: &str) -> bool {
    let l = line.to_ascii_lowercase();
    l.contains("error") || l.contains("validationerror")
}

pub fn gmail_clip_warning(html_bytes: u64) -> Option<String> {
    if html_bytes >= 100_000 {
        Some(format!(
            "Compiled HTML is {html_bytes} bytes. Gmail clips near 100KB."
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("dd_emailforge_mjml_{}_{}", std::process::id(), n));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_fake_mjml(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, "#!/bin/sh\necho fake-mjml\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).unwrap();
        }
    }

    #[test]
    fn discover_prefers_local_node_modules() {
        let dir = temp_dir();
        let local = dir.join("node_modules/.bin/mjml");
        write_fake_mjml(&local);
        let found = discover_mjml(&dir).unwrap();
        assert_eq!(found, local);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn discover_not_found_lists_searched() {
        let dir = temp_dir();
        match discover_mjml(&dir) {
            Err(MjmlError::NotFound { searched }) => {
                assert!(
                    searched
                        .iter()
                        .any(|s| s.contains("node_modules/.bin/mjml"))
                );
                assert!(searched.iter().any(|s| s == "$PATH"));
            }
            other => panic!("expected NotFound, got {other:?}"),
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn flags_do_not_include_allow_includes_or_dash_l() {
        // Lock the argv contract in a helper snapshot of the flag list.
        let mut cmd = Command::new("mjml");
        push_mjml_flags(&mut cmd);
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let joined = args.join(" ");
        assert!(joined.contains("validationLevel"));
        assert!(joined.contains("strict"));
        assert!(!joined.contains("allowIncludes"));
        assert!(!args.iter().any(|a| a == "-l"));
        assert!(joined.contains("allowMixedSyntax"));
        assert!(joined.contains("templateSyntax"));
    }

    #[test]
    #[ignore = "requires official mjml CLI"]
    fn mjml_strict_compiles_minimal() {
        let dir = temp_dir();
        let mjml_src = dir.join("template.mjml");
        fs::write(
            &mjml_src,
            "<mjml><mj-body><mj-section><mj-column><mj-text>Hi</mj-text></mj-column></mj-section></mj-body></mjml>\n",
        )
        .unwrap();
        let bin = discover_mjml(&dir).expect("mjml on PATH");
        let out = dir.join("template.html");
        compile_one_shot_captured(&bin, &dir, &mjml_src, &out).unwrap();
        let html = fs::read_to_string(&out).unwrap();
        assert!(html.to_ascii_lowercase().contains("<!doctype html") || html.contains("<html"));
        let _ = fs::remove_dir_all(&dir);
    }
}
