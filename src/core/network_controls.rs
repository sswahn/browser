use native_tls::{TlsConnector, TlsStream};
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const HTTP_PORT: u16 = 80;
const HTTPS_PORT: u16 = 443;
const HTTPS_PREFIX: &str = "https://";
const HTTP_PREFIX: &str = "http://";

enum BrowserError {
    InvalidUrlFormat { host: String, path: String },
    ConnectionError,
    TlsError(native_tls::Error),
    WorkingStreamError,
    HandleRequestError(std::io::Error),
}

struct ConnectionPool {
    pool: Arc<Mutex<HashMap<String, TcpStream>>>,
}

impl ConnectionPool {
    fn new() -> Self {
        ConnectionPool {
            pool: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn get_connection(&self, host: &str, port: u16) -> Result<TcpStream, BrowserError> {
        let key = format!("{}:{}", host, port);
        let mut pool = self.pool.lock().unwrap();

        if let Some(stream) = pool.get(&key) {
            Ok(stream.try_clone().expect("Failed to clone TCP stream"))
        } else {
            let new_stream = TcpStream::connect(&key).await.map_err(|_| BrowserError::ConnectionError)?;

            pool.insert(key.clone(), new_stream.try_clone().expect("Failed to clone TCP stream"));

            Ok(pool[&key].try_clone().expect("Failed to clone TCP stream"))
        }
    }
}

struct HttpResponse {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

async fn http_response(url: &str) -> Result<Response, BrowserError> {
    let host = parse_url(&url);
    let port = get_port(&url);
    let stream = connect_to_stream(&host, port).await?;
    make_request(&stream, &host)
}

fn parse_url(url: &str) -> String {
    let url = url.trim_start_matches(HTTP_PREFIX).trim_start_matches(HTTPS_PREFIX);
    let (host, path) = url.split_once('/').unwrap_or((url, ""));
    validate_url(&host, &path);
    host.to_string()
}

fn get_port(url: &str) -> u16 {
    if url.starts_with(HTTPS_PREFIX) { HTTPS_PORT } else { HTTP_PORT }
}

async fn connect_to_stream(host: &str, port: u16) -> TcpStream {
    pool.get_connection(host, port)
    //TcpStream::connect(format!("{}:{}", host, port))
}

async fn make_request(stream: &mut TcpStream, host: &str) -> Result<String, BrowserError> {
    let working_stream = get_working_stream(&host, &stream).await?;
    handle_request(&working_stream, &host)
}

async fn get_working_stream(host: &str, stream: &mut TcpStream) -> Result<TcpStream, BrowserError> {
    if host.starts_with(HTTPS_PREFIX) {
        upgrade_to_https(host, stream).await.map_err(|err| BrowserError::TlsError(err))
    } else {
        Ok(stream)
    }
}

async fn upgrade_to_https(host: &str, stream: &TcpStream) -> Result<TlsStream<TcpStream>, native_tls::Error> {
    let connector = TlsConnector::new()?;
    let tls_stream = connector.connect(host, stream)?;
    Ok(tls_stream)
}

async fn handle_request(stream: &TcpStream, host: &str) -> Result<String, BrowserError> {
    let request = format!("GET / HTTP/2.0\r\nHost: {}\r\nUser-Agent: Browser\r\n\r\n", host);
    stream.write_all(request.as_bytes()).await?;
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;
    Ok(String::from_utf8_lossy(&buffer).to_string())
}

fn validate_url(host: &str, path: &str) -> Result<(), BrowserError> {
    if host.is_empty() || path.is_empty() {
        Err(BrowserError::InvalidUrlFormat {
            host: host.to_string(),
            path: path.to_string(),
        })
    } else {
        Ok(())
    }
}

fn parse_http_response(response: &str) -> Option<HttpResponse> {
    let mut lines = response.lines();

    // Parse status line
    if let Some(status_line) = lines.next() {
        let (status, _, _) = parse_status_line(status_line)?;

        // Check if status is in the success range
        if status >= 200 && status < 300 {
            let mut headers = Vec::new();

            // Parse headers
            while let Some(line) = lines.next() {
                if line.trim().is_empty() {
                    break;
                }

                if let Some((name, value)) = parse_header(line) {
                    headers.push((name, value));
                }
            }

            // Parse body
            let body = lines.collect::<Vec<&str>>().join("\n");

            Some(HttpResponse {
                status,
                headers,
                body,
            })
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_status_line(status_line: &str) -> Option<(u16, &str, &str)> {
    let mut parts = status_line.split_whitespace().collect::<Vec<&str>>();
    if parts.len() >= 3 {
        let status = parts[0].parse().ok()?;
        let version = parts[1];
        let reason = parts[2..].join(" ");
        Some((status, version, &reason))
    } else {
        None
    }
}

fn parse_header(header_line: &str) -> Option<(String, String)> {
    let mut parts = header_line.splitn(2, ':');
    if let (Some(name), Some(value)) = (parts.next(), parts.next()) {
        let name = name.trim().to_string();
        let value = value.trim().to_string();
        Some((name, value))
    } else {
        None
    }
}
