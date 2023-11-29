use native_tls::{TlsConnector, TlsStream};
use std::net::TcpStream;
use std::str;
use std::sync::Mutex;
use tokio::task;

const HTTP_PORT: u16 = 80;
const HTTPS_PORT: u16 = 443;
const HTTPS_PREFIX: &str = "https://";
const HTTP_PREFIX: &str = "http://";

enum BrowserError {
    InvalidUrlFormat { host: String, path: String },
    TlsError(Box<dyn std::error::Error>),
}

fn get_port(url: &str) -> u16 {
    if url.starts_with(HTTPS_PREFIX) { HTTPS_PORT } else { HTTP_PORT }
}

async fn connect_to_stream(host: &str, port: u16) -> TcpStream {
    TcpStream::connect(format!("{}:{}", host, port))
}

async fn make_request(stream: &mut TcpStream, host: &str) {
    let working_stream = if host.starts_with(HTTPS_PREFIX) {
        upgrade_to_https(host, stream).await.unwrap()
    } else {
        stream
    };
    handle_request(&working_stream, host)
}

async fn upgrade_to_https(host: &str, stream: &mut TcpStream) -> Result<TlsStream<TcpStream>, BrowserError> {
    let connector = TlsConnector::new().map_err(|e| BrowserError::TlsError(Box::new(e)))?;
    let tls_stream = connector.connect(host, stream).map_err(|e| BrowserError::TlsError(Box::new(e)))?;
    Ok(tls_stream)
}

async fn handle_request(stream: &TcpStream, host: &str) -> Result<String, Box<dyn std::error::Error>> {
    let request = format!("GET / HTTP/2.0\r\nHost: {}\r\nUser-Agent: Browser\r\n\r\n", host);
    stream.write_all(request.as_bytes()).await?;
    let mut buffer = Vec::new();
    stream.take(1024).read_to_end(&mut buffer).await?;
    Ok(String::from_utf8_lossy(&buffer).to_string())
}

fn parse_url(url: &str) -> (String, String) {
    let url = url.trim_start_matches(HTTP_PREFIX).trim_start_matches(HTTPS_PREFIX);
    let (host, path) = url.split_once('/').unwrap_or((url, ""));
    validate_url(&host, &path);
    (host.to_string(), path.to_string())
}

fn validate_url(host: &str, path: &str) -> Result<(), BrowserError> {
    if host.is_empty() || path.is_empty() {
        return Err(BrowserError::InvalidUrlFormat {
            host: host.to_string(),
            path: path.to_string(),
        });
    }
    Ok(())
}

fn parse_http_response(response: &str) -> Option<(String, String)> {
    let mut lines = response.lines();
    if let Some(status_line) = lines.next() {
        let (status, _version, _reason) = parse_status_line(status_line);
        if status >= 200 && status < 300 {
            let mut headers = String::new();
            while let Some(line) = lines.next() {
                if line.trim().is_empty() {
                    break;
                }
                headers.push_str(line);
                headers.push('\n');
            }
            let body = lines.collect::<Vec<&str>>().join("\n");
            Some((headers, body))
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_status_line(status_line: &str) -> Result<(u16, &str, &str), &'static str> {
    let mut parts = status_line.split_whitespace().collect::<Vec<&str>>();
    if parts.len() >= 3 {
        let status = parts[0].parse().unwrap_or(0);
        let version = parts[1];
        let reason = parts[2..].join(" ");
        Ok((status, version, &reason))
    } else {
        Err("Invalid status line format")
    }
}
