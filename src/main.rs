use native_tls::{TlsConnector, TlsStream};
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::str;

const HTTP_PORT: u16 = 80;
const HTTPS_PORT: u16 = 443;

fn main() {
    let url = read_user_input("Enter URL: ");
    let (host, path) = parse_url(&url);
    let port = get_port(&url);
    validate_url(&url, &host, &path);
    let stream = connect_to_stream(&host, port);
    handle_request(&mut stream, &url);
}

fn connect_to_stream(host: &str, port: u16) -> Result<TcpStream, std::io::Error> {
    TcpStream::connect(format!("{}:{}", host, port))
}

fn handle_request(stream: &mut Result<TcpStream, std::io::Error>, host: &str, path: &str) {
    match stream {
        Ok(mut stream) => {
            if host.starts_with("https://") {
                // Upgrade the connection to HTTPS if needed
                handle_tls_stream(&mut stream, host, path);
            } else {
                // Handle HTTP request
                handle_http_request(&mut stream, host, path);
            }
        },
        Err(_) => {
            eprintln!("Failed to connect to the host")
        },
    }
}

fn handle_tls_stream(stream: &mut TcpStream, host: &str, path: &str) {
    if let Ok(tls_stream) = upgrade_to_https(host, stream) {
        handle_https_request(&tls_stream, host, &path);
    } else {
        eprintln!("Failed to establish a secure connection");
    }
}

fn validate_url(url: &str, host: &str, path: &str) -> Result<(), &'static str> {
    if parse_url(url).is_none() {
        eprintln!("Invalid URL format");
        return Err("Invalid URL format");
    }
    Ok(())
}

fn get_port(url: &str) -> u16 {
    if url.starts_with("https://") { HTTPS_PORT } else { HTTP_PORT }
}

fn read_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("Failed to flush standard output");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string();
}

fn parse_url(url: &str) -> (String, String) {
    let url = url.trim_start_matches("http://").trim_start_matches("https://");
    let (host, path) = url.split_once('/').unwrap_or(("", ""));
    (host.to_string(), path.to_string())
}

fn upgrade_to_https(host: &str, stream: TcpStream) -> Result<TlsStream<TcpStream>, native_tls::Error> {
    let connector = TlsConnector::new()?;
    let tls_stream = connector.connect(host, stream)?;
    Ok(tls_stream)
}

fn handle_http_request(stream: &mut TcpStream, host: &str, path: &str) {
    let request = format!("GET {} HTTP/2.0\r\nHost: {}\r\n\r\n", path, host);
    write_to_stream(stream, &request);
    let response = read_from_stream(stream);
    println!("{}", response);
}

fn handle_https_request(tls_stream: &TlsStream<TcpStream>, host: &str, path: &str) {
    let request = format!("GET {} HTTP/2.0\r\nHost: {}\r\n\r\n", path, host);
    write_to_stream(tls_stream, &request);
    let response = read_from_stream(tls_stream);
    println!("{}", response);
}

fn write_to_stream<S: Write>(stream: &mut S, data: &str) {
    stream.write(data.as_bytes()).expect("Failed to write to stream");
}

fn read_from_stream<S: Read>(stream: &mut S) -> String {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).expect("Failed to read from stream");
    String::from_utf8_lossy(&buffer).to_string();
}
