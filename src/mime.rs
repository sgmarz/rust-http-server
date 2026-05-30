use std::{fs::File, path::Path, io::Read};

pub fn mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") | Some("htm") => "text/html",
        Some("css") => "text/css",
        Some("js") | Some("mjs") => "application/javascript",
        Some("rs") => "text/plain",
        Some("hs") => "text/plain",
        Some("cpp") => "text/plain",
        Some("c") => "text/plain",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("wasm") => "application/wasm",
        Some("pdf") => "application/pdf",
        Some("txt") => "text/plain",
        Some("xml") => "application/xml",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        _ => determine_mime_type(path),
    }
}

fn determine_mime_type(path: &Path) -> &'static str {
    if let Ok(fl) = File::open(path) {
        for b in fl.bytes() {
            let byte = b.unwrap_or(0);
            if !byte.is_ascii() {
                return "application/octet-stream";
            }
        }
        "text/plain"
    }
    else {
        "application/octet-stream"
    }
}

