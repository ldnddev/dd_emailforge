//! Loopback preview wrapper (subject/preheader chrome + 600/320 iframes).
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::UNIX_EPOCH;

use serde::Serialize;

use crate::mjml::MjmlWatch;

#[derive(Debug, Clone, Serialize)]
pub struct PreviewMeta {
    pub subject: String,
    pub preheader: String,
}

pub struct PreviewSession {
    _join: JoinHandle<()>,
    pub watch: Option<MjmlWatch>,
    pub port: u16,
    pub meta: Arc<Mutex<PreviewMeta>>,
}

struct HttpState {
    template_root: PathBuf,
    compiled: PathBuf,
    meta: Arc<Mutex<PreviewMeta>>,
}

pub fn start_http(
    template_root: PathBuf,
    compiled: PathBuf,
    meta: Arc<Mutex<PreviewMeta>>,
    bind: &str,
) -> std::io::Result<(u16, JoinHandle<()>)> {
    let listener = TcpListener::bind(bind)?;
    let port = listener.local_addr()?.port();
    let state = Arc::new(HttpState {
        template_root,
        compiled,
        meta,
    });
    let handle = std::thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                let state = Arc::clone(&state);
                std::thread::spawn(move || {
                    let _ = handle_client(stream, &state);
                });
            }
        }
    });
    Ok((port, handle))
}

impl PreviewSession {
    pub fn start_tui(
        template_root: PathBuf,
        compiled: PathBuf,
        meta: PreviewMeta,
        watch: Option<MjmlWatch>,
    ) -> std::io::Result<Self> {
        let meta = Arc::new(Mutex::new(meta));
        let (port, join) = start_http(template_root, compiled, Arc::clone(&meta), "127.0.0.1:0")?;
        Ok(Self {
            _join: join,
            watch,
            port,
            meta,
        })
    }

    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}/", self.port)
    }
}

const CSP_WRAPPER: &str = "default-src 'self'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; img-src 'self'; frame-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";
const CSP_COMPILED: &str = "default-src 'none'; img-src 'self' https: data:; style-src 'unsafe-inline'; font-src https://fonts.gstatic.com https://fonts.googleapis.com data:; base-uri 'none'; form-action 'none'; frame-ancestors 'self'";
const CSP_ASSET: &str = "default-src 'none'; script-src 'none'; style-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";
const CSP_DATA: &str = "default-src 'none'; frame-ancestors 'none'";

fn handle_client(mut stream: TcpStream, state: &HttpState) -> std::io::Result<()> {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Ok(());
    }
    let req = String::from_utf8_lossy(&buf[..n]);
    if !host_is_loopback(&req) {
        return write_response(
            &mut stream,
            "400 Bad Request",
            b"Bad Request",
            "text/plain; charset=utf-8",
            CSP_DATA,
        );
    }
    let Some(path) = parse_path(&req).and_then(percent_decode) else {
        return write_response(
            &mut stream,
            "400 Bad Request",
            b"Bad Request",
            "text/plain; charset=utf-8",
            CSP_DATA,
        );
    };
    let (status, body, ctype, csp) = route(&path, state);
    write_response(&mut stream, status, &body, ctype, csp)
}

fn write_response(
    stream: &mut TcpStream,
    status: &str,
    body: &[u8],
    ctype: &str,
    csp: &str,
) -> std::io::Result<()> {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: {csp}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    Ok(())
}

/// HTTP/1.1 Host must be the loopback name the server bound. A foreign Host is
/// how a DNS-rebinding page reads a predictable preview port from the browser.
fn host_is_loopback(req: &str) -> bool {
    let mut saw = false;
    for line in req.lines().skip(1) {
        if line.is_empty() {
            break;
        }
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if !name.eq_ignore_ascii_case("host") {
            continue;
        }
        saw = true;
        let host = value.trim();
        let hostname = host.split(':').next().unwrap_or(host).trim();
        if hostname != "127.0.0.1" && !hostname.eq_ignore_ascii_case("localhost") {
            return false;
        }
    }
    saw
}

fn percent_decode(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() {
                return None;
            }
            let hi = hex_nibble(bytes[i + 1])?;
            let lo = hex_nibble(bytes[i + 2])?;
            out.push((hi << 4) | lo);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    if out.contains(&0) {
        return None;
    }
    String::from_utf8(out).ok()
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn parse_path(req: &str) -> Option<&str> {
    let line = req.lines().next()?;
    let mut parts = line.split_whitespace();
    let method = parts.next()?;
    if method != "GET" && method != "HEAD" {
        return None;
    }
    let target = parts.next()?;
    Some(target.split('?').next().unwrap_or(target))
}

fn route(path: &str, state: &HttpState) -> (&'static str, Vec<u8>, &'static str, &'static str) {
    if path.components_parent() {
        return not_found();
    }
    match path {
        "/" => {
            let meta = state.meta.lock().map(|g| g.clone()).unwrap_or(PreviewMeta {
                subject: String::new(),
                preheader: String::new(),
            });
            (
                "200 OK",
                wrapper_html(&meta).into_bytes(),
                "text/html; charset=utf-8",
                CSP_WRAPPER,
            )
        }
        "/compiled.html" => {
            if let Ok(bytes) = std::fs::read(&state.compiled) {
                ("200 OK", bytes, "text/html; charset=utf-8", CSP_COMPILED)
            } else {
                (
                    "200 OK",
                    b"<!doctype html><title>compiling</title><p>compiling...</p>".to_vec(),
                    "text/html; charset=utf-8",
                    CSP_COMPILED,
                )
            }
        }
        "/__mtime" => {
            let secs = std::fs::metadata(&state.compiled)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            (
                "200 OK",
                secs.to_string().into_bytes(),
                "text/plain; charset=utf-8",
                CSP_DATA,
            )
        }
        "/__meta" => {
            let meta = state.meta.lock().map(|g| g.clone()).unwrap_or(PreviewMeta {
                subject: String::new(),
                preheader: String::new(),
            });
            let json = serde_json::to_string(&meta).unwrap_or_else(|_| "{}".into());
            ("200 OK", json.into_bytes(), "application/json", CSP_DATA)
        }
        p if p.starts_with("/images/") => serve_image(&state.template_root, p),
        _ => not_found(),
    }
}

fn not_found() -> (&'static str, Vec<u8>, &'static str, &'static str) {
    (
        "404 Not Found",
        b"Not Found".to_vec(),
        "text/plain; charset=utf-8",
        CSP_DATA,
    )
}

trait ParentDirCheck {
    fn components_parent(&self) -> bool;
}

impl ParentDirCheck for str {
    fn components_parent(&self) -> bool {
        Path::new(self)
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    }
}

fn serve_image(root: &Path, url_path: &str) -> (&'static str, Vec<u8>, &'static str, &'static str) {
    let rel = url_path.trim_start_matches('/');
    let rel_path = Path::new(rel);
    // Only plain path segments. Parent, current, root, and prefix components
    // are rejected before the filesystem is touched.
    let normal = rel.starts_with("images/")
        && rel_path
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_)));
    if !normal {
        return not_found();
    }
    let images = root.join("images");
    // `images/` itself must stay inside the template root (a symlink of the
    // directory to `/` would otherwise become the confinement root).
    let Some(images_root) = crate::paths::resolved_path_within(root, &images) else {
        return not_found();
    };
    let candidate = root.join(rel_path);
    let Some(resolved) = crate::paths::resolved_path_within(root, &candidate) else {
        return not_found();
    };
    if !resolved.starts_with(&images_root) || !resolved.is_file() {
        return not_found();
    }
    match std::fs::read(&resolved) {
        Ok(bytes) => ("200 OK", bytes, mime_for(&resolved), CSP_ASSET),
        Err(_) => not_found(),
    }
}

pub fn mime_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

pub fn wrapper_html(meta: &PreviewMeta) -> String {
    let subject = html_escape(&meta.subject);
    let preheader = html_escape(&meta.preheader);
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>dd_emailforge preview</title>
<style>
  body {{ margin:0; background:#111; color:#eee; font:14px/1.4 system-ui,sans-serif; }}
  header {{ padding:12px 16px; border-bottom:1px solid #333; }}
  .k {{ color:#888; font-size:11px; text-transform:uppercase; }}
  .canvases {{ display:flex; gap:24px; padding:16px; align-items:flex-start; overflow:auto; }}
  figure {{ margin:0; }}
  iframe {{ background:#fff; border:0; height:80vh; display:block; }}
  figcaption {{ text-align:center; margin-top:8px; color:#888; }}
</style>
</head>
<body>
<header>
  <div class="k">Subject</div>
  <div id="subject">{subject}</div>
  <div class="k" style="margin-top:8px">Preheader</div>
  <div id="preheader">{preheader}</div>
</header>
<div class="canvases">
  <figure>
    <iframe id="desk" src="/compiled.html" width="600" sandbox="allow-popups allow-popups-to-escape-sandbox"></iframe>
    <figcaption>600px</figcaption>
  </figure>
  <figure>
    <iframe id="mob" src="/compiled.html" width="320" sandbox="allow-popups allow-popups-to-escape-sandbox"></iframe>
    <figcaption>320px</figcaption>
  </figure>
</div>
<script>
let lastMtime = null;
function applyMeta(data) {{
  document.getElementById('subject').textContent = data.subject || '';
  document.getElementById('preheader').textContent = data.preheader || '';
}}
async function tick() {{
  try {{
    const m = await fetch('/__mtime').then(r => r.text());
    if (lastMtime !== null && m !== lastMtime) {{
      const bust = '/compiled.html?t=' + m;
      document.getElementById('desk').src = bust;
      document.getElementById('mob').src = bust;
    }}
    lastMtime = m;
    const meta = await fetch('/__meta').then(r => r.json());
    applyMeta(meta);
  }} catch (e) {{}}
}}
setInterval(tick, 700);
</script>
</body>
</html>
"#
    )
}

pub fn wrapper_markers() -> &'static [&'static str] {
    &["/__mtime", "compiled.html?t=", "iframe"]
}

pub fn html_contains_wrapper_markers(html: &str) -> bool {
    wrapper_markers().iter().any(|m| html.contains(m))
}

pub fn wait_for_interrupt() {
    #[cfg(unix)]
    {
        static STOP: AtomicBool = AtomicBool::new(false);
        extern "C" fn handler(_: i32) {
            STOP.store(true, Ordering::SeqCst);
        }
        unsafe {
            unsafe extern "C" {
                fn signal(sig: i32, handler: extern "C" fn(i32)) -> usize;
            }
            signal(2, handler);
        }
        while !STOP.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }
    #[cfg(not(unix))]
    {
        let mut line = String::new();
        let _ = std::io::stdin().read_line(&mut line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("dd_emailforge_prev_{}_{}", std::process::id(), n));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn http_get(port: u16, path: &str) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        stream
            .write_all(
                format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
                    .as_bytes(),
            )
            .unwrap();
        let mut buf = String::new();
        stream.read_to_string(&mut buf).unwrap();
        buf
    }

    #[test]
    fn wrapper_escapes_subject_on_get_slash() {
        let dir = temp_dir();
        let compiled = dir.join(".preview/template.html");
        std::fs::create_dir_all(compiled.parent().unwrap()).unwrap();
        std::fs::write(&compiled, b"<p>hi</p>").unwrap();
        let meta = Arc::new(Mutex::new(PreviewMeta {
            subject: "Hello <b>&\"x".into(),
            preheader: "pre <em>".into(),
        }));
        let (port, _h) = start_http(dir.clone(), compiled, meta, "127.0.0.1:0").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        let body = http_get(port, "/");
        assert!(body.contains("Hello &lt;b&gt;&amp;&quot;x"));
        assert!(body.contains("pre &lt;em&gt;"));
        assert!(!body.contains("Hello <b>"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn meta_is_json_not_xml_escaped() {
        let dir = temp_dir();
        let compiled = dir.join(".preview/template.html");
        std::fs::create_dir_all(compiled.parent().unwrap()).unwrap();
        std::fs::write(&compiled, b"<p>hi</p>").unwrap();
        let meta = Arc::new(Mutex::new(PreviewMeta {
            subject: "A & B".into(),
            preheader: "p".into(),
        }));
        let live = Arc::clone(&meta);
        let (port, _h) = start_http(dir.clone(), compiled, meta, "127.0.0.1:0").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        let body = http_get(port, "/__meta");
        let json = body.split("\r\n\r\n").nth(1).unwrap_or(&body);
        let v: serde_json::Value = serde_json::from_str(json).expect("json");
        assert_eq!(v["subject"], "A & B");
        {
            let mut g = live.lock().unwrap();
            g.subject = "updated".into();
        }
        let body2 = http_get(port, "/__meta");
        let json2 = body2.split("\r\n\r\n").nth(1).unwrap_or(&body2);
        let v2: serde_json::Value = serde_json::from_str(json2).unwrap();
        assert_eq!(v2["subject"], "updated");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn blocks_parent_dir() {
        let dir = temp_dir();
        let compiled = dir.join(".preview/template.html");
        std::fs::create_dir_all(compiled.parent().unwrap()).unwrap();
        std::fs::write(&compiled, b"<p>hi</p>").unwrap();
        let meta = Arc::new(Mutex::new(PreviewMeta {
            subject: "s".into(),
            preheader: String::new(),
        }));
        let (port, _h) = start_http(dir.clone(), compiled, meta, "127.0.0.1:0").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        let denied = http_get(port, "/../Cargo.toml");
        assert!(denied.contains("Not Found") || denied.contains("404"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_and_encoded_dotdot_are_not_served() {
        let dir = temp_dir();
        let secret = dir.join("secret.txt");
        std::fs::write(&secret, b"topsecret").unwrap();
        let images = dir.join("images");
        std::fs::create_dir_all(&images).unwrap();
        std::fs::write(images.join("ok.txt"), b"visible").unwrap();
        std::os::unix::fs::symlink(&secret, images.join("leak.txt")).unwrap();
        let compiled = dir.join(".preview/template.html");
        std::fs::create_dir_all(compiled.parent().unwrap()).unwrap();
        std::fs::write(&compiled, b"<p>hi</p>").unwrap();
        let meta = Arc::new(Mutex::new(PreviewMeta {
            subject: "s".into(),
            preheader: String::new(),
        }));
        let (port, _h) = start_http(dir.clone(), compiled, meta, "127.0.0.1:0").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        let leaked = http_get(port, "/images/leak.txt");
        assert!(leaked.contains("404") || leaked.contains("Not Found"));
        assert!(!leaked.contains("topsecret"));
        let encoded = http_get(port, "/images/%2e%2e/secret.txt");
        assert!(encoded.contains("404") || encoded.contains("Not Found"));
        assert!(!encoded.contains("topsecret"));
        let ok = http_get(port, "/images/ok.txt");
        assert!(ok.contains("visible"));
        let foreign = http_get_host(port, "/images/ok.txt", "evil.example");
        assert!(foreign.starts_with("HTTP/1.1 400"));
        assert!(!foreign.contains("visible"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn http_get_host(port: u16, path: &str, host: &str) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        stream
            .write_all(
                format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n")
                    .as_bytes(),
            )
            .unwrap();
        let mut buf = String::new();
        stream.read_to_string(&mut buf).unwrap();
        buf
    }

    #[test]
    fn compiled_missing_is_compiling_page() {
        let dir = temp_dir();
        let compiled = dir.join(".preview/template.html");
        std::fs::create_dir_all(compiled.parent().unwrap()).unwrap();
        let meta = Arc::new(Mutex::new(PreviewMeta {
            subject: "s".into(),
            preheader: String::new(),
        }));
        let (port, _h) = start_http(dir.clone(), compiled, meta, "127.0.0.1:0").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        let body = http_get(port, "/compiled.html");
        assert!(body.contains("compiling"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn wrapper_html_contains_markers_emit_does_not() {
        let html = wrapper_html(&PreviewMeta {
            subject: "s".into(),
            preheader: "p".into(),
        });
        assert!(html_contains_wrapper_markers(&html));
        let t = crate::model::Template::minimal();
        let mjml = crate::emit::emit_mjml(&t, crate::emit::EmitMode::Export).unwrap();
        assert!(!html_contains_wrapper_markers(&mjml));
    }
}
