//! Client connection handling and response generation.
//! Stephen Marz
//! 5-Jun-2026
use crate::{
    args::Args,
    http::{ParseError, Request, Response, parse_request, response_name},
    mime,
};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tokio::{
    fs,
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};
use tokio_rustls::server::TlsStream;

/// Entry point for a single accepted connection.
pub async fn handle(stream: TcpStream, addr: SocketAddr, args: Args) {
    let mut reader = BufReader::new(stream);

    let request = match parse_request(&mut reader).await {
        Ok(r) => r,
        Err(ParseError::Eof) => return, // client disconnected cleanly
        Err(e) => {
            eprintln!("[{addr}] parse error: {e}");
            return;
        }
    };

    let response = build_response(&request, &args).await;

    if !args.quiet && !args.silent {
        println!(
            "{addr} \"{} {}\" ({} {}) ({} bytes).",
            request.method,
            request.path,
            response.status,
            response_name(response.status),
            response.body.len(),
        );
    }

    let bytes = response.into_bytes();
    let stream = reader.into_inner();
    let mut stream = stream;

    if let Err(e) = stream.write_all(&bytes).await {
        eprintln!("[{addr}] write error: {e}");
    }
}

/// Entry point for a single accepted connection.
pub async fn handle_tls(stream: TlsStream<TcpStream>, addr: SocketAddr, args: Args) {
    let mut reader = BufReader::new(stream);

    let request = match parse_request(&mut reader).await {
        Ok(r) => r,
        Err(ParseError::Eof) => return, // client disconnected cleanly
        Err(e) => {
            eprintln!("[{addr}] parse error: {e}");
            return;
        }
    };

    let response = build_response(&request, &args).await;

    if !args.quiet && !args.silent {
        println!(
            "{addr} \"{} {}\" ({} {}) ({} bytes).",
            request.method,
            request.path,
            response.status,
            response_name(response.status),
            response.body.len(),
        );
    }

    let bytes = response.into_bytes();
    let stream = reader.into_inner();
    let mut stream = stream;

    if let Err(e) = stream.write_all(&bytes).await {
        eprintln!("[{addr}] write error: {e}");
    }
    if let Err(e) = stream.shutdown().await {
        eprintln!("[{addr}] TLS shutdown error: {e}");
    }
}

/// Build an HTTP response. This produces the response
/// that can be sent later.
async fn build_response(req: &Request, args: &Args) -> Response {
    // Only GET is supported for a static file server.
    if req.method != "GET" && req.method != "HEAD" {
        return Response::method_not_allowed();
    }

    // Decode percent-encoded characters and strip query string.
    let decoded = percent_decode(req.path.split('?').next().unwrap_or("/"));

    // Prevent path traversal: canonicalize and verify the path stays inside root.
    let rel = decoded.trim_start_matches('/');
    let candidate = args.root.join(rel);

    let resolved = match resolve_safe(&args.root, &candidate).await {
        Some(p) => p,
        None => return Response::forbidden(),
    };

    if resolved.is_dir() {
        // Try index.html first.
        let index = resolved.join("index.html");
        if !args.no_index && index.is_file() {
            serve_file(&index, args.cache).await
        } else if !args.no_dir_listing {
            serve_directory(&resolved, &decoded, args.cache).await
        } else {
            Response::forbidden()
        }
    } else if resolved.is_file() {
        serve_file(&resolved, args.cache).await
    } else {
        Response::not_found()
    }
}

/// Ensure `path` is within `root` after canonicalization. This prevents
/// injection attacks.
async fn resolve_safe(root: &Path, path: &Path) -> Option<PathBuf> {
    // Canonicalize root once; we use a simpler lexical check if fs fails.
    let canon_root = fs::canonicalize(root).await.ok()?;

    // If the target doesn't exist yet we can't canonicalize it, so normalize
    // manually and check the prefix.
    let normalized = normalize_path(path);
    let canon_path = if normalized.exists() {
        fs::canonicalize(&normalized).await.ok()?
    } else {
        normalized
    };
    if canon_path.starts_with(&canon_root) {
        Some(canon_path)
    } else {
        None
    }
}

/// Lexically normalize a path (resolve `.` and `..`) without touching the FS.
fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        use std::path::Component::{CurDir, ParentDir};
        match component {
            ParentDir => {
                out.pop();
            }
            CurDir => {}
            c => out.push(c),
        }
    }
    out
}

// ── File Serving ─────────────────────────────────────────────────────────────

async fn serve_file(path: &Path, cache_max_age: u64) -> Response {
    match fs::read(path).await {
        Ok(bytes) => Response::ok(bytes, mime::mime_type(path), cache_max_age),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => Response::forbidden(),
        Err(e) if e.kind() == std::io::ErrorKind::OutOfMemory => Response::too_large(),
        Err(_) => Response::not_found(),
    }
}

// ── Directory Listing ─────────────────────────────────────────────────────────

async fn serve_directory(dir: &Path, url_path: &str, cache_max_age: u64) -> Response {
    let mut entries = match fs::read_dir(dir).await {
        Ok(e) => e,
        Err(_) => return Response::not_found(),
    };

    let mut rows = Vec::new();

    // Parent link (unless we're at root "/")
    if url_path != "/" {
        let parent = parent_url(url_path);
        rows.push(format!(r#"<li><a href="{parent}">../</a></li>"#));
    }

    // Collect entries and sort: dirs first, then files, both alphabetical.
    let mut items: Vec<(String, bool)> = Vec::new(); // (name, is_dir)
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
        items.push((name, is_dir));
    }
    items.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    for (name, is_dir) in items {
        let display = if is_dir {
            format!("{name}/")
        } else {
            name.clone()
        };
        let href = format!("{}/{}", url_path.trim_end_matches('/'), name);
        let href = if is_dir { format!("{href}/") } else { href };
        rows.push(format!(r#"<li><a href="{href}">{display}</a></li>"#));
    }

    let listing = rows.join("\n        ");
    let body = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Index of {url_path}</title>
  <style>
    body {{ font-family: monospace; padding: 2em; }}
    ul   {{ list-style: none; padding: 0; }}
    li   {{ padding: 0.2em 0; }}
    a    {{ text-decoration: none; color: #0066cc; }}
    a:hover {{ text-decoration: underline; }}
  </style>
</head>
<body>
  <h1>Index of {url_path}</h1>
  <ul>
        {listing}
  </ul>
</body>
</html>"#
    );

    Response::ok(body.into_bytes(), "text/html", cache_max_age)
}

fn parent_url(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    match trimmed.rfind('/') {
        Some(0) | None => "/".to_owned(),
        Some(i) => trimmed[..i].to_owned(),
    }
}

// ── Percent-decoding ─────────────────────────────────────────────────────────

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
