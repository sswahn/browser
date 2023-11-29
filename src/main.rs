use native_tls::{TlsConnector, TlsStream};
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::str;

const HTTP_PORT: u16 = 80;
const HTTPS_PORT: u16 = 443;

struct Browser {
    history: VecDeque<String>,
    current_url: Option<String>,
    cache: HashMap<String, String>,
}

impl Browser {
    fn new() -> Self {
        Browser {
            history: VecDeque::new(),
            current_url: None,
            cache: HashMap::new(),
        }
    }

    fn navigate(&mut self, url: String) {
        self.history.push_back(url.clone());
        self.current_url = Some(url);
    }

    fn back(&mut self) -> Option<String> {
        self.history.pop_back().map(|url| {
            self.current_url = self.history.back().cloned();
            url
        })
    }

    fn forward(&mut self) -> Option<String> {
        self.history.pop_front().map(|url| {
            self.current_url = self.history.front().cloned();
            url
        })
    }

    fn refresh(&mut self) {
        if let Some(url) = self.current_url.clone() {
            self.cache.remove(&url);
        }
    }

    fn set_cache(&mut self, url: &str, response: String) {
        self.cache.insert(url.to_string(), response);
    }

    fn get_cache(&self, url: &str) -> Option<&String> {
        self.cache.get(url)
    }
}

fn main() {
    let mut browser = Browser::new();
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
        make_request(&mut stream, &url);
    }
}

async fn make_request(stream: &mut Result<TcpStream, std::io::Error>, host: &str, path: &str) {
    match stream {
        Ok(mut stream) => {
            if let Some(cached_response) = browser.get_cached(url) {
                println!("Using cached response for: {}", url);
                render_html(cached_response);
            } else {
                handle_request(&mut stream, url, &mut browser.cache).await?;
            }

            
            if host.starts_with("https://") {
                handle_tls_stream(&mut stream, host, path);
            } else {
                handle_request(&mut stream, host, path);
            }
        },
        Err(_) => {
            eprintln!("Failed to connect to the host")
        },
    }
}

async fn connect_to_stream(host: &str, port: u16) -> Result<TcpStream, std::io::Error> {
    TcpStream::connect(format!("{}:{}", host, port)).map_err(|e| {
        eprintln!("Failed to connect to the host: {}", e);
        e
    });
}

fn get_port(url: &str) -> u16 {
    if url.starts_with("https://") { HTTPS_PORT } else { HTTP_PORT }
}

fn parse_url(url: &str) -> (String, String) {
    let url = url.trim_start_matches("http://").trim_start_matches("https://");
    let (host, path) = url.split_once('/').unwrap_or((url, ""));
    validate_url(&url, &host, &path);
    (host.to_string(), path.to_string())
}

fn validate_url(url: &str, host: &str, path: &str) -> Result<(), &'static str> {
    let (parsed_host, _) = parse_url(url);
    if parsed_host.is_empty() || path.is_empty() {
        eprintln!("Invalid URL format");
        return Err("Invalid URL format");
    }
    Ok(())
}

fn upgrade_to_https(host: &str, stream: TcpStream) -> Result<TlsStream<TcpStream>, native_tls::Error> {
    let connector = TlsConnector::new()?;
    let tls_stream = connector.connect(host, stream)?;
    Ok(tls_stream);
}

async fn handle_tls_stream(stream: &mut TcpStream, host: &str, path: &str) {
    if let Ok(tls_stream) = upgrade_to_https(host, stream) {
        handle_request(&tls_stream, host, &path);
    } else {
        eprintln!("Failed to establish a secure connection");
    }
}

async fn handle_request(stream: &mut TcpStream, host: &str, path: &str) {
    let request = format!("GET {} HTTP/2.0\r\nHost: {}\r\nUser-Agent: Browser\r\n\r\n", path, host);
    if let Err(e) = write_to_stream(stream, &request).await {
        return eprintln!("Failed to write to stream: {}", e);
    }
    let response = read_from_stream(stream).await;
    if let Some((headers, body)) = parse_http_response(&response) {
        println!("Headers:\n{}", headers);
        println!("Body:\n{}", body);
        render_html(&body)
        // set cache
    } else {
        eprintln!("Failed to parse HTTP response");
    }
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

fn parse_status_line(status_line: &str) -> (u16, &str, &str) {
    let mut parts = status_line.split_whitespace();
    let status = parts.next().unwrap_or("0").parse().unwrap_or(0);
    let version = parts.next().unwrap_or("");
    let reason = parts.skip(1).collect::<Vec<&str>>().join(" ");
    (status, version, &reason)
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

