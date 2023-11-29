use native_tls::{TlsConnector, TlsStream};
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::str;
mod core/browser

const HTTP_PORT: u16 = 80;
const HTTPS_PORT: u16 = 443;

fn main() {
    let mut browser = browser::Browser::new();
    loop {
        let input = read_user_input("Enter URL: ");
        let url = input.trim().to_string();
        browser.navigate(url.clone());
        let cached_response = browser.get_cache(&url)
        if Some(cached_response) {
            return render_html(cached_response);
        }
        let (host, path) = parse_url(&url);
        let port = get_port(&url);
        let stream = connect_to_stream(&host, port).await;
        let response = make_request(&mut stream, &host, &url).await;
        render_html(&response)
        browser.set_cache(&url, &response)
    }
}

fn get_port(url: &str) -> u16 {
    if url.starts_with("https://") { HTTPS_PORT } else { HTTP_PORT }
}

async fn connect_to_stream(host: &str, port: u16) -> TcpStream {
    TcpStream::connect(format!("{}:{}", host, port))
}

async fn make_request(stream: &mut TcpStream, host: &str, path: &str) {
    if host.starts_with("https://") {
        handle_tls_stream(&mut stream, host, path);
    } else {
        handle_request(&mut stream, host, path);
    }
}

async fn handle_tls_stream(stream: &mut TcpStream, host: &str, path: &str) {
    let tls_stream = upgrade_to_https(host, stream);
    handle_request(&tls_stream, host, &path);
}

fn upgrade_to_https(host: &str, stream: &mut TcpStream) -> TlsStream<TcpStream> {
    let connector = TlsConnector::new();
    connector.connect(host, stream);
}

async fn handle_request(stream: &mut TcpStream, host: &str, path: &str) {
    let request = format!("GET {} HTTP/2.0\r\nHost: {}\r\nUser-Agent: Browser\r\n\r\n", path, host);
    write_to_stream(stream, &request).await;
    let response = read_from_stream(stream).await;
    let (headers, body) = parse_http_response(&response);
    body;
}

fn parse_url(url: &str) -> (String, String) {
    let url = url.trim_start_matches("http://").trim_start_matches("https://");
    let (host, path) = url.split_once('/').unwrap_or((url, ""));
    validate_url(&host, &path);
    (host.to_string(), path.to_string())
}

fn validate_url(host: &str, path: &str) -> Result<(), &'static str> {
    if host.is_empty() || path.is_empty() {
        eprintln!("Invalid URL format");
        return Err("Invalid URL format");
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

fn read_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("Failed to flush standard output");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string();
}

async fn write_to_stream<S: Write>(stream: &mut S, data: &str) {
    stream.write(data.as_bytes()).expect("Failed to write to stream");
}

async fn read_from_stream<S: Read>(stream: &mut S) -> String {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).expect("Failed to read from stream");
    String::from_utf8_lossy(&buffer).to_string();
}

fn render_html(html: &str) {
    let mut in_tag = false;
    let mut tag_name = String::new();
    let mut in_text = false;

    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
                tag_name.clear();
            }
            '>' => {
                in_tag = false;
                if tag_name.starts_with('/') {
                    // Closing tag, ignore for simplicity
                } else if tag_name.ends_with('/') {
                    // Self-closing tag, ignore for simplicity
                } else {
                    // Opening tag, print tag name
                    println!("Start tag: {}", tag_name);
                }
            }
            '/' if in_tag => {
                // Inside a tag, this is a potential closing tag
                // Ignore for simplicity
            }
            'a'..='z' | 'A'..='Z' if in_tag => {
                // Collect characters to get tag name
                tag_name.push(c);
            }
            _ => {
                if !in_tag {
                    // Inside text, print text content
                    print!("{}", c);
                }
            }
        }
    }
}
