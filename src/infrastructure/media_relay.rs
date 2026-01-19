//! Local media relay server.
//!
//! Serves local media files over HTTP for the Dioxus desktop WebView.
//! Supports byte-range requests for video seeking.
//
//! Example:
//! let relay = MediaRelayServer::start()?;
//! let url = relay.media_url("/absolute/path/to/video.mp4");
//! // pass `url` to the <video> src
//! // relay.stop()? when shutting down

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

/// Error type for media relay.
#[derive(Debug, thiserror::Error)]
pub enum MediaRelayError {
    #[error("I/O error: {0}")]
    Io(String),
    #[error("Server thread failed")]
    Thread,
}

/// Local media relay server.
pub struct MediaRelayServer {
    base_url: String,
    shutdown_tx: Sender<()>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MediaRelayServer {
    /// Start the relay server on localhost with an ephemeral port.
    pub fn start() -> Result<Self, MediaRelayError> {
        let listener =
            TcpListener::bind("127.0.0.1:0").map_err(|e| MediaRelayError::Io(e.to_string()))?;
        listener.set_nonblocking(true).map_err(|e| MediaRelayError::Io(e.to_string()))?;

        let addr = listener.local_addr().map_err(|e| MediaRelayError::Io(e.to_string()))?;
        let base_url = format!("http://127.0.0.1:{}", addr.port());

        let (tx, rx) = mpsc::channel::<()>();

        let handle = thread::spawn(move || {
            run_server(listener, rx);
        });

        Ok(Self { base_url, shutdown_tx: tx, handle: Some(handle) })
    }

    /// Base URL of the relay server (e.g., http://127.0.0.1:12345).
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Build a full media URL for a local file path.
    pub fn media_url(&self, path: &str) -> String {
        let encoded = url_encode(path);
        format!("{}/media?path={}", self.base_url, encoded)
    }

    /// Stop the relay server.
    pub fn stop(mut self) -> Result<(), MediaRelayError> {
        let _ = self.shutdown_tx.send(());
        if let Some(handle) = self.handle.take() {
            handle.join().map_err(|_| MediaRelayError::Thread)?;
        }
        Ok(())
    }
}

impl Drop for MediaRelayServer {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(());
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn run_server(listener: TcpListener, shutdown_rx: Receiver<()>) {
    loop {
        if shutdown_rx.try_recv().is_ok() {
            break;
        }

        match listener.accept() {
            Ok((mut stream, _)) => {
                let _ = handle_connection(&mut stream);
            },
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(16));
            },
            Err(_) => break,
        }
    }
}

fn handle_connection(stream: &mut TcpStream) -> std::io::Result<()> {
    let request = read_request(stream)?;
    if request.is_empty() {
        return Ok(());
    }

    let (method, target, headers) = parse_request(&request);
    if method != "GET" && method != "HEAD" {
        return write_response(stream, 405, "text/plain", b"Method Not Allowed");
    }

    if !target.starts_with("/media") {
        return write_response(stream, 404, "text/plain", b"Not Found");
    }

    let Some(path_param) = parse_query_param(&target, "path") else {
        return write_response(stream, 400, "text/plain", b"Missing path parameter");
    };

    let decoded = url_decode(&path_param).unwrap_or(path_param);
    let path = PathBuf::from(decoded);

    let path = match canonicalize_path(&path) {
        Ok(p) => p,
        Err(_) => return write_response(stream, 404, "text/plain", b"File not found"),
    };

    if !path.is_file() {
        return write_response(stream, 404, "text/plain", b"File not found");
    }

    let mut file = File::open(&path)?;
    let size = file.metadata()?.len();

    let content_type = content_type_for_path(&path);
    let range = headers.get("range").and_then(|v| parse_range(v, size));

    match range {
        Some(RangeResult::Valid { start, end }) => {
            let length = end - start + 1;
            file.seek(SeekFrom::Start(start))?;
            let mut buffer = vec![0u8; length as usize];
            file.read_exact(&mut buffer)?;

            let mut response = Vec::new();
            response.extend_from_slice(
                format!(
                    "HTTP/1.1 206 Partial Content\r\nContent-Type: {}\r\nContent-Length: {}\r\nContent-Range: bytes {}-{}/{}\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n",
                    content_type, length, start, end, size
                )
                .as_bytes(),
            );

            if method == "GET" {
                response.extend_from_slice(&buffer);
            }

            stream.write_all(&response)?;
        },
        Some(RangeResult::Invalid) => {
            let response = format!(
                "HTTP/1.1 416 Range Not Satisfiable\r\nContent-Range: bytes */{}\r\nConnection: close\r\n\r\n",
                size
            );
            stream.write_all(response.as_bytes())?;
        },
        None => {
            let mut response = Vec::new();
            response.extend_from_slice(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n",
                    content_type, size
                )
                .as_bytes(),
            );

            if method == "GET" {
                let mut buffer = Vec::with_capacity(size as usize);
                file.read_to_end(&mut buffer)?;
                response.extend_from_slice(&buffer);
            }

            stream.write_all(&response)?;
        },
    }

    Ok(())
}

fn read_request(stream: &mut TcpStream) -> std::io::Result<String> {
    let mut buffer = [0u8; 8192];
    let mut collected = Vec::new();

    loop {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        collected.extend_from_slice(&buffer[..read]);
        if collected.windows(4).any(|w| w == b"\r\n\r\n") || collected.len() > 64 * 1024 {
            break;
        }
    }

    Ok(String::from_utf8_lossy(&collected).to_string())
}

fn parse_request(request: &str) -> (String, String, std::collections::HashMap<String, String>) {
    let mut lines = request.lines();
    let request_line = lines.next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("").to_string();

    let mut headers = std::collections::HashMap::new();
    for line in lines {
        if line.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_lowercase(), v.trim().to_string());
        }
    }

    (method, target, headers)
}

fn parse_query_param(target: &str, key: &str) -> Option<String> {
    let query = target.split_once('?')?.1;
    for pair in query.split('&') {
        let mut it = pair.splitn(2, '=');
        let k = it.next()?;
        let v = it.next().unwrap_or("");
        if k == key {
            return Some(v.to_string());
        }
    }
    None
}

fn url_encode(input: &str) -> String {
    input
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                (b as char).to_string()
            },
            _ => format!("%{:02X}", b),
        })
        .collect()
}

fn url_decode(input: &str) -> Option<String> {
    let mut out = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let h1 = bytes[i + 1];
                let h2 = bytes[i + 2];
                let hex = [h1, h2];
                let value = u8::from_str_radix(std::str::from_utf8(&hex).ok()?, 16).ok()?;
                out.push(value);
                i += 3;
            },
            b'+' => {
                out.push(b' ');
                i += 1;
            },
            b => {
                out.push(b);
                i += 1;
            },
        }
    }
    String::from_utf8(out).ok()
}

fn content_type_for_path(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase().as_str() {
        "mp4" => "video/mp4",
        "mkv" => "video/x-matroska",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

fn canonicalize_path(path: &Path) -> std::io::Result<PathBuf> {
    std::fs::canonicalize(path)
}

enum RangeResult {
    Valid { start: u64, end: u64 },
    Invalid,
}

fn parse_range(header_value: &str, size: u64) -> Option<RangeResult> {
    let value = header_value.trim().to_lowercase();
    if !value.starts_with("bytes=") {
        return None;
    }

    let range = value.trim_start_matches("bytes=").trim();
    let (start_str, end_str) = range.split_once('-').unwrap_or((range, ""));

    let start = if start_str.is_empty() { None } else { start_str.parse::<u64>().ok() };
    let end = if end_str.is_empty() { None } else { end_str.parse::<u64>().ok() };

    match (start, end) {
        (Some(s), Some(e)) if s <= e && e < size => Some(RangeResult::Valid { start: s, end: e }),
        (Some(s), None) if s < size => Some(RangeResult::Valid { start: s, end: size - 1 }),
        (None, Some(e)) if e > 0 => {
            let len = e.min(size);
            let start = size.saturating_sub(len);
            Some(RangeResult::Valid { start, end: size - 1 })
        },
        _ => Some(RangeResult::Invalid),
    }
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    let status_line = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        416 => "Range Not Satisfiable",
        _ => "OK",
    };

    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status,
        status_line,
        content_type,
        body.len()
    );

    stream.write_all(response.as_bytes())?;
    stream.write_all(body)?;
    Ok(())
}
