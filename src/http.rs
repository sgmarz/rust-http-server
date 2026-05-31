use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;

// ── Request ──────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
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
        if let ParseError::Io(e) = self {
            Some(e)
        }
        else {
            None
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
pub async fn parse_request(stream: &mut BufReader<TcpStream>) -> Result<Request, ParseError> {
    // Read the request line (e.g. "GET /index.html HTTP/1.1")
    let mut request_line = String::new();
    let n = stream.read_line(&mut request_line).await?;
    if n == 0 {
        return Err(ParseError::Eof);
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().ok_or(ParseError::BadRequestLine)?.to_owned();
    let path = parts.next().ok_or(ParseError::BadRequestLine)?.to_owned();

    // Drain headers so the stream is ready for the next request.
    loop {
        let mut header_line = String::new();
        stream.read_line(&mut header_line).await?;
        if header_line == "\r\n" || header_line == "\n" || header_line.is_empty() {
            break;
        }
    }

    Ok(Request { method, path })
}

// ── Response ─────────────────────────────────────────────────────────────────

pub struct Response {
    pub status: u16,
    pub reason: &'static str,
    pub content_type: &'static str,
    pub cache_max_age: u64,
    pub body: Vec<u8>,
}

impl Response {
    pub fn ok(body: Vec<u8>, content_type: &'static str, cache_max_age: u64) -> Self {
        Self {
            status: 200,
            reason: "OK",
            content_type,
            cache_max_age,
            body,
        }
    }

    pub fn not_found() -> Self {
        let body = b"<html><body><h1>404 Not Found</h1></body></html>".to_vec();
        Self {
            status: 404,
            reason: "Not Found",
            content_type: "text/html",
            cache_max_age: 0,
            body,
        }
    }

    pub fn forbidden() -> Self {
        let body = b"<html><body><h1>403 Forbidden</h1></body></html>".to_vec();
        Self {
            status: 403,
            reason: "Forbidden",
            content_type: "text/html",
            cache_max_age: 0,
            body,
        }
    }

    pub fn method_not_allowed() -> Self {
        let body = b"<html><body><h1>405 Method Not Allowed</h1></body></html>".to_vec();
        Self {
            status: 405,
            reason: "Method Not Allowed",
            content_type: "text/html",
            cache_max_age: 0,
            body,
        }
    }

    pub fn too_large() -> Self {
        let body = b"<html><body><h1>413 Content Too Large</h1></body></html>".to_vec();
        Self {
            status: 413,
            reason: "Content Too Large",
            content_type: "text/html",
            cache_max_age: 0,
            body,
        }
    }

    /// Serialise the response into a byte buffer ready to write to the socket.
    pub fn into_bytes(self) -> Vec<u8> {
        let cache_header = if self.cache_max_age > 0 {
            format!("Cache-Control: max-age={}\r\n", self.cache_max_age)
        }
        else {
            "Cache-Control: no-store\r\n".to_owned()
        };

        let header = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}; charset=utf-8\r\nContent-Length: {}\r\n{}Connection: close\r\n\r\n",
            self.status,
            self.reason,
            self.content_type,
            self.body.len(),
            cache_header,
        );

        let mut buf = header.into_bytes();
        buf.extend_from_slice(&self.body);
        buf
    }
}
