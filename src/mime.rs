use std::{fs::File, io::Read, path::Path};

pub fn mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") | Some("htm") => "text/html",
        Some("css") => "text/css",
        Some("js") | Some("mjs") => "application/javascript",
        Some("adoc") => "text/asciidoc",
        Some("json") => "application/json",
        Some("toml") => "text/plain",
        Some("php") => "application/x-httpd-php",
        Some("md") => "text/markdown",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("bmp") => "image/bmp",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("wasm") => "application/wasm",
        Some("pdf") => "application/pdf",
        Some("xml") => "application/xml",
        Some("wav") => "audio/wav",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("bz2") => "application/x-bzip2",
        Some("gz") => "application/gzip",
        Some("zip") => "application/zip",
        Some("7z") => "application/x-7z-compressed",
        Some("rtf") => "application/rtf",
        Some("doc") => "application/msword",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("ppt") => "application/vnd.ms-powerpoint",
        Some("pptx") => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        Some("xls") => "application/vnd.ms-excel",
        Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        _ => determine_mime_type(path),
    }
}
const ASCII_FAILURES: usize = 25;
fn determine_mime_type(path: &Path) -> &'static str {
    if let Ok(fl) = File::open(path) {
        for b in fl.bytes().take(ASCII_FAILURES) {
            let byte = b.unwrap_or(0);
            if !byte.is_ascii() {
                return "application/octet-stream";
            }
        }
        return "text/plain";
    }
    "application/octet-stream"
}
