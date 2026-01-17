//! Local YouTube embed relay server.
//!
//! Purpose: Serve a local HTTP origin (http://127.0.0.1:<port>) so the WebView
//! includes a valid Referer/Origin when YouTube iframes load.
//!
//! This is designed for desktop WebView environments where custom schemes
//! (e.g., dioxus://) cause YouTube Error 153 due to missing referrer.
//!
//! Usage:
//! ```ignore
//! let relay = EmbedRelayServer::start()?;
//! let url = relay.embed_url("dQw4w9WgXcQ");
//! // pass `url` to the iframe src
//! // relay.stop()? when shutting down
//! ```

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum EmbedRelayError {
    #[error("I/O error: {0}")]
    Io(String),
    #[error("Server thread failed")]
    Thread,
}

pub struct EmbedRelayServer {
    base_url: String,
    shutdown_tx: Sender<()>,
    handle: Option<thread::JoinHandle<()>>,
}

impl EmbedRelayServer {
    /// Start the relay server on localhost with an ephemeral port.
    pub fn start() -> Result<Self, EmbedRelayError> {
        let listener =
            TcpListener::bind("127.0.0.1:0").map_err(|e| EmbedRelayError::Io(e.to_string()))?;
        listener.set_nonblocking(true).map_err(|e| EmbedRelayError::Io(e.to_string()))?;

        let addr = listener.local_addr().map_err(|e| EmbedRelayError::Io(e.to_string()))?;
        let base_url = format!("http://127.0.0.1:{}", addr.port());

        let (tx, rx) = mpsc::channel::<()>();

        let handle = thread::spawn(move || {
            run_loop(listener, rx);
        });

        Ok(Self { base_url, shutdown_tx: tx, handle: Some(handle) })
    }

    /// Base URL of the relay server (e.g., http://127.0.0.1:12345).
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Build a full embed URL for a YouTube video ID.
    pub fn embed_url(&self, video_id: &str) -> String {
        let safe = sanitize_video_id(video_id);
        format!("{}/embed?v={}", self.base_url, safe)
    }

    /// Stop the relay server.
    pub fn stop(mut self) -> Result<(), EmbedRelayError> {
        let _ = self.shutdown_tx.send(());
        if let Some(handle) = self.handle.take() {
            handle.join().map_err(|_| EmbedRelayError::Thread)?;
        }
        Ok(())
    }
}

impl Drop for EmbedRelayServer {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(());
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn run_loop(listener: TcpListener, shutdown_rx: Receiver<()>) {
    loop {
        if shutdown_rx.try_recv().is_ok() {
            break;
        }

        match listener.accept() {
            Ok((mut stream, _)) => {
                let _ = handle_connection(&mut stream, listener.local_addr());
            },
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            },
            Err(_) => {
                thread::sleep(Duration::from_millis(50));
            },
        }
    }
}

fn handle_connection(
    stream: &mut TcpStream,
    addr: std::io::Result<std::net::SocketAddr>,
) -> Result<(), std::io::Error> {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Ok(());
    }

    let req = String::from_utf8_lossy(&buf[..n]);
    let mut lines = req.lines();
    let request_line = lines.next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("");

    if method != "GET" {
        return write_response(stream, 405, "text/plain; charset=utf-8", "Method Not Allowed");
    }

    if target == "/health" {
        return write_response(stream, 200, "text/plain; charset=utf-8", "ok");
    }

    if target.starts_with("/embed") {
        let v = parse_query_param(target, "v");
        let Some(video_id) = v else {
            return write_response(stream, 400, "text/plain; charset=utf-8", "Missing v parameter");
        };

        let origin = addr
            .ok()
            .map(|a| format!("http://127.0.0.1:{}", a.port()))
            .unwrap_or_else(|| "http://127.0.0.1".to_string());

        let html = build_embed_html(&sanitize_video_id(&video_id), &origin);
        return write_response(stream, 200, "text/html; charset=utf-8", &html);
    }

    write_response(stream, 404, "text/plain; charset=utf-8", "Not Found")
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
) -> Result<(), std::io::Error> {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "OK",
    };

    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\n\r\n{}",
        status,
        status_text,
        content_type,
        body.len(),
        body
    );

    stream.write_all(response.as_bytes())
}

fn build_embed_html(video_id: &str, origin: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="referrer" content="strict-origin-when-cross-origin" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<style>
html, body {{ height: 100%; margin: 0; background: #000; }}
iframe {{ width: 100%; height: 100%; border: 0; display: block; }}
</style>
</head>
<body>
<iframe
  src="https://www.youtube.com/embed/{video_id}?rel=0&modestbranding=1&playsinline=1&origin={origin}"
  allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
  allowfullscreen
  referrerpolicy="strict-origin-when-cross-origin">
</iframe>
</body>
</html>"#,
        video_id = video_id,
        origin = origin
    )
}

fn parse_query_param(target: &str, key: &str) -> Option<String> {
    let mut split = target.splitn(2, '?');
    let _path = split.next();
    let query = split.next()?;
    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        let k = kv.next().unwrap_or_default();
        let v = kv.next().unwrap_or_default();
        if k == key {
            return Some(percent_decode(v));
        }
    }
    None
}

fn percent_decode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.as_bytes().iter().copied();
    while let Some(c) = chars.next() {
        if c == b'%' {
            let hi = chars.next();
            let lo = chars.next();
            if let (Some(hi), Some(lo)) = (hi, lo) {
                if let (Some(h), Some(l)) = (hex_val(hi), hex_val(lo)) {
                    out.push((h * 16 + l) as char);
                    continue;
                }
            }
            out.push('%');
        } else if c == b'+' {
            out.push(' ');
        } else {
            out.push(c as char);
        }
    }
    out
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(10 + (b - b'a')),
        b'A'..=b'F' => Some(10 + (b - b'A')),
        _ => None,
    }
}

fn sanitize_video_id(video_id: &str) -> String {
    video_id.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-').collect()
}
