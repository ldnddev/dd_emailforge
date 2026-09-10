mod border;
mod emit;
mod mjml;
mod model;
mod padding;
mod paths;
mod preview;
mod starters;
mod storage;
mod tui;
mod validate;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

use starters::StarterKind;

use storage::{LoadError, load_template, resolve_template_path};
use validate::validate_template_with_root;

#[derive(Debug, Parser)]
#[command(
    name = "dd_emailforge",
    version,
    about = "Terminal-UI email template builder"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create a template folder (JSON, images/, package.json, .gitignore).
    Init {
        dir: PathBuf,
        /// Starter: welcome | newsletter | promo | transactional (default: welcome).
        #[arg(long, value_enum, default_value_t = StarterKind::Welcome)]
        from: StarterKind,
    },
    /// Open the TUI on a template.json or a folder containing one.
    Tui { path: Option<PathBuf> },
    /// Structural + image validation. Non-zero exit on errors. Warnings on stderr.
    Validate { path: PathBuf },
    /// Pretty-print the loaded template JSON to stdout.
    Show { path: PathBuf },
    /// Validate, emit MJML, compile HTML with official mjml.
    Export {
        path: PathBuf,
        /// Destination directory (default: the template folder).
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Validate, emit, start mjml -w + loopback wrapper, open the browser.
    Preview {
        path: PathBuf,
        #[arg(long, default_value_t = 8766)]
        port: u16,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Init { dir, from } => cmd_init(&dir, from),
        Command::Tui { path } => match tui::run_tui(path) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e:#}");
                ExitCode::from(1)
            }
        },
        Command::Validate { path } => cmd_validate(&path),
        Command::Show { path } => cmd_show(&path),
        Command::Export { path, out } => cmd_export(&path, out.as_deref()),
        Command::Preview { path, port } => cmd_preview(&path, port),
    }
}

fn cmd_init(dir: &PathBuf, from: StarterKind) -> ExitCode {
    match starters::init_template_dir(dir, from) {
        Ok(json) => {
            println!("Created {}", dir.display());
            println!("Wrote {}", json.display());
            println!("Wrote {}", dir.join("package.json").display());
            println!("{}", starters::init_next_steps(dir));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e:#}");
            ExitCode::from(1)
        }
    }
}

fn cmd_validate(path: &PathBuf) -> ExitCode {
    let json_path = match resolve_template_path(path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e:#}");
            return ExitCode::from(1);
        }
    };
    let template = match load_cli(&json_path) {
        Ok(t) => t,
        Err(code) => return code,
    };
    let root = storage::template_root(&json_path);
    let report = validate_template_with_root(&template, Some(&root));
    for err in &report.errors {
        eprintln!("- {err}");
    }
    for warn in &report.warnings {
        eprintln!("warning: {warn}");
    }
    if report.ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn cmd_show(path: &PathBuf) -> ExitCode {
    let json_path = match resolve_template_path(path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e:#}");
            return ExitCode::from(1);
        }
    };
    let template = match load_cli(&json_path) {
        Ok(t) => t,
        Err(code) => return code,
    };
    match serde_json::to_string_pretty(&template) {
        Ok(mut s) => {
            if !s.ends_with('\n') {
                s.push('\n');
            }
            print!("{s}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("failed to serialize template: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_export(path: &PathBuf, out: Option<&std::path::Path>) -> ExitCode {
    let json_path = match resolve_template_path(path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e:#}");
            return ExitCode::from(1);
        }
    };
    let template = match load_cli(&json_path) {
        Ok(t) => t,
        Err(code) => return code,
    };
    let root = storage::template_root(&json_path);
    let out_dir = out.unwrap_or(&root);
    if let Err(e) = std::fs::create_dir_all(out_dir) {
        eprintln!("could not create {}: {e}", out_dir.display());
        return ExitCode::from(1);
    }
    let report = crate::validate::validate_template_for_export(&template, Some(&root));
    for err in &report.errors {
        eprintln!("- {err}");
    }
    for warn in &report.warnings {
        eprintln!("warning: {warn}");
    }
    if !report.ok() {
        return ExitCode::from(1);
    }
    let bin = match mjml::discover_mjml(&root) {
        Ok(p) => p,
        Err(e) => {
            eprint!("{e}");
            return ExitCode::from(1);
        }
    };
    let mjml_path = out_dir.join("template.mjml");
    let html_path = out_dir.join("template.html");
    if let Err(e) = crate::emit::write_mjml(&template, &mjml_path, crate::emit::EmitMode::Export) {
        eprintln!("{e:#}");
        return ExitCode::from(1);
    }
    match mjml::compile_one_shot_captured(&bin, &root, &mjml_path, &html_path) {
        Ok(result) => {
            println!("Wrote {}", mjml_path.display());
            println!("Wrote {}", html_path.display());
            if let Some(w) = mjml::gmail_clip_warning(result.html_bytes) {
                eprintln!("warning: {w}");
            }
            if crate::preview::html_contains_wrapper_markers(
                &std::fs::read_to_string(&html_path).unwrap_or_default(),
            ) {
                eprintln!("warning: compiled HTML unexpectedly contains preview wrapper markers");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprint!("{e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_preview(path: &PathBuf, port: u16) -> ExitCode {
    let json_path = match resolve_template_path(path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e:#}");
            return ExitCode::from(1);
        }
    };
    let template = match load_cli(&json_path) {
        Ok(t) => t,
        Err(code) => return code,
    };
    let root = storage::template_root(&json_path);
    let report = validate_template_with_root(&template, Some(&root));
    for err in &report.errors {
        eprintln!("- {err}");
    }
    for warn in &report.warnings {
        eprintln!("warning: {warn}");
    }
    if !report.ok() {
        return ExitCode::from(1);
    }
    let bin = match mjml::discover_mjml(&root) {
        Ok(p) => p,
        Err(e) => {
            eprint!("{e}");
            return ExitCode::from(1);
        }
    };
    let preview_dir = root.join(".preview");
    if let Err(e) = std::fs::create_dir_all(&preview_dir) {
        eprintln!("{e}");
        return ExitCode::from(1);
    }
    let compiled = preview_dir.join("template.html");
    let meta = std::sync::Arc::new(std::sync::Mutex::new(preview::PreviewMeta {
        subject: template.subject.clone(),
        preheader: template.preheader.clone(),
    }));
    let bind = format!("127.0.0.1:{port}");
    let (bound, _join) = match preview::start_http(root.clone(), compiled.clone(), meta, &bind) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("preview server: {e}");
            return ExitCode::from(1);
        }
    };
    let origin = format!("http://127.0.0.1:{bound}");
    let mjml_path = root.join("template.mjml");
    if let Err(e) = crate::emit::write_mjml(
        &template,
        &mjml_path,
        crate::emit::EmitMode::Preview {
            origin: origin.clone(),
        },
    ) {
        eprintln!("{e:#}");
        return ExitCode::from(1);
    }
    let _watch = match mjml::MjmlWatch::spawn(&bin, &root, &mjml_path, &compiled) {
        Ok(w) => w,
        Err(e) => {
            eprint!("{e}");
            return ExitCode::from(1);
        }
    };
    let url = format!("http://127.0.0.1:{bound}/");
    eprintln!("Serving wrapper at {url}");
    let _ = tui_open_browser_cli(&url);
    preview::wait_for_interrupt();
    ExitCode::SUCCESS
}

fn tui_open_browser_cli(url: &str) -> std::io::Result<()> {
    crate::tui_open(url)
}

fn tui_open(url: &str) -> std::io::Result<()> {
    use std::process::{Command, Stdio};
    let mut cmd;
    #[cfg(target_os = "linux")]
    {
        cmd = Command::new("xdg-open");
        cmd.arg(url);
    }
    #[cfg(target_os = "macos")]
    {
        cmd = Command::new("open");
        cmd.arg(url);
    }
    #[cfg(target_os = "windows")]
    {
        cmd = Command::new("cmd");
        cmd.args(["/C", "start", ""]).arg(url);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = url;
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

fn load_cli(path: &std::path::Path) -> Result<crate::model::Template, ExitCode> {
    match load_template(path) {
        Ok(t) => Ok(t),
        Err(LoadError::MissingVersion) => {
            eprintln!("missing template.json version (expected 1)");
            Err(ExitCode::from(2))
        }
        Err(LoadError::UnsupportedVersion(n)) => {
            eprintln!("unsupported template.json version {n} (expected 1)");
            Err(ExitCode::from(2))
        }
        Err(LoadError::Parse(msg)) => {
            eprintln!("failed to parse template.json: {msg}");
            Err(ExitCode::from(2))
        }
        Err(LoadError::Io(e)) => {
            eprintln!("failed to read template.json: {e}");
            Err(ExitCode::from(2))
        }
    }
}
