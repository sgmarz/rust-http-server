//! HTTP protocol parsing and response building.
//! Stephen Marz
//! 5-Jun-2026
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt};

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
}

impl Request {
    pub fn new(method: &str, path: &str) -> Self {
        Self {
            method: method.to_owned(),
            path: path.to_owned(),
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    Io(std::io::Error),
    BadRequestLine,
    Eof,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Io(e) => write!(f, "I/O error: {e}"),
            ParseError::BadRequestLine => write!(f, "malformed request line"),
            ParseError::Eof => write!(f, "connection closed before request"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(e: std::io::Error) -> Self {
        ParseError::Io(e)
    }
}

// We hand-roll the parser to avoid external deps.  Only the request line is
// needed for a static file server; headers are drained but not stored.
pub async fn parse_request<R: AsyncBufRead + AsyncReadExt + Unpin>(
    stream: &mut R,
) -> Result<Request, ParseError> {
    // Read the request line (e.g. "GET /index.html HTTP/1.1")
    let mut request_line = String::new();

    // Since HTTP is line-oriented, we need BufReader.
    match stream.read_line(&mut request_line).await? {
        // We only care if this fails. We can detect this by getting 0 bytes, which
        // means that the client closed the connection before sending a request.
        0 => return Err(ParseError::Eof),
        _ => {}
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().ok_or(ParseError::BadRequestLine)?;
    let path = parts.next().ok_or(ParseError::BadRequestLine)?;

    // Drain headers so the stream is ready for the next request.
    loop {
        let mut header_line = String::new();
        stream.read_line(&mut header_line).await?;
        if header_line == "\r\n" || header_line == "\n" || header_line.is_empty() {
            break;
        }
    }

    Ok(Request::new(method, path))
}

// ── Response ─────────────────────────────────────────────────────────────────

pub struct Response {
    pub status: u16,
    pub reason: &'static str,
    pub content_type: &'static str,
    pub cache_max_age: u64,
    pub body: Vec<u8>,
    pub location: Option<String>,
}

impl Response {
    pub fn ok(body: Vec<u8>, content_type: &'static str, cache_max_age: u64) -> Self {
        Self {
            status: 200,
            reason: "OK",
            content_type,
            cache_max_age,
            body,
            location: None,
        }
    }

    pub fn redirect(location: &str) -> Self {
        let body = format!(
            "<html><head><title>301 Moved Permanently</title></head><body><h1>301 Moved Permanently</h1><p>Redirecting to <a href=\"{location}\">{location}</a></p></body></html>"
        ).into_bytes();
        // We can't use the normal into_bytes() because we need a Location header,
        // so carry it as extra metadata or just build the raw bytes here.
        Self {
            status: 301,
            reason: "Moved Permanently",
            content_type: "text/html",
            cache_max_age: 0,
            body,
            location: Some(location.to_owned()),
        }
    }

    pub fn not_found() -> Self {
        let status = 404;
        Self {
            status,
            reason: "Not Found",
            content_type: "text/html",
            cache_max_age: 0,
            body: make_body(status),
            location: None,
        }
    }

    pub fn forbidden() -> Self {
        let status = 403;
        Self {
            status,
            reason: "Forbidden",
            content_type: "text/html",
            cache_max_age: 0,
            body: make_body(status),
            location: None,
        }
    }

    pub fn method_not_allowed() -> Self {
        let status = 405;
        Self {
            status,
            reason: "Method Not Allowed",
            content_type: "text/html",
            cache_max_age: 0,
            body: make_body(status),
            location: None,
        }
    }

    pub fn too_large() -> Self {
        let status = 413;
        Self {
            status,
            reason: "Content Too Large",
            content_type: "text/html",
            cache_max_age: 0,
            body: make_body(status),
            location: None,
        }
    }

    /// Serialise the response into a byte buffer ready to write to the socket.
    pub fn into_bytes(self) -> Vec<u8> {
        let cache_header = if self.cache_max_age > 0 {
            format!("Cache-Control: max-age={}\r\n", self.cache_max_age)
        }
        else {
            String::from("Cache-Control: no-store\r\n")
        };
        let location_header = match &self.location {
            Some(url) => format!("Location: {url}\r\n"),
            None => String::new(),
        };

        let header = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}; charset=utf-8\r\nContent-Length: {}\r\n{}{}Connection: close\r\n\r\n",
            self.status,
            self.reason,
            self.content_type,
            self.body.len(),
            cache_header,
            location_header,
        );

        let mut buf = header.into_bytes();
        buf.extend_from_slice(&self.body);
        buf
    }
}

/// Map status codes to reason phrases for logging or debugging.
pub fn response_name(code: u16) -> &'static str {
    match code {
        200 => "OK",
        301 => "Moved Permanently",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Content Too Large",
        _ => "Unknown Status",
    }
}

pub fn make_body(code: u16) -> Vec<u8> {
    let name = response_name(code);
    format!(
        "<html><head><title>{code} {name}</title></head><body><h1>{code} {name}</h1></body></html>"
    )
    .into_bytes()
}
