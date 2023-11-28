use native_tls::{TlsConnector, TlsStream};
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::str;

const HTTP_PORT: u16 = 80;
const HTTPS_PORT: u16 = 443;

fn main() {
    let url = read_user_input("Enter URL: ");
    let (host, path) = parse_url(&url);

    // Determine the appropriate port based on the scheme
    let port = if url.starts_with("https://") { HTTPS_PORT } else { HTTP_PORT };

    // Connect to the host
    match TcpStream::connect(format!("{}:{}", host, port)) {
        Ok(mut stream) => {
            if url.starts_with("https://") {
                // Upgrade the connection to HTTPS if needed
                if let Ok(tls_stream) = upgrade_to_https(&host, stream) {
                    handle_https_request(&tls_stream, &host, &path);
                } else {
                    eprintln!("Failed to establish a secure connection");
                }
            } else {
                // Handle HTTP request
                handle_http_request(&mut stream, &host, &path);
            }
        }
        Err(_) => eprintln!("Failed to connect to the host"),
    }
}

fn read_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("Failed to flush stdout");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

fn parse_url(url: &str) -> (String, &str) {
    let url = url.trim_start_matches("http://").trim_start_matches("https://");
    let (host, path) = url.split_once('/').unwrap_or(("", ""));
    (host.to_string(), path)
}

fn upgrade_to_https(host: &str, stream: TcpStream) -> Result<TlsStream<TcpStream>, native_tls::Error> {
    let connector = TlsConnector::new()?;
    connector.connect(host, stream)
}

fn handle_http_request(stream: &mut TcpStream, host: &str, path: &str) {
    let request = format!("GET {} HTTP/1.0\r\nHost: {}\r\n\r\n", path, host);
    write_to_stream(stream, &request);
    let response = read_from_stream(stream);
    println!("{}", response);
}

fn handle_https_request(tls_stream: &TlsStream<TcpStream>, host: &str, path: &str) {
    let request = format!("GET {} HTTP/1.0\r\nHost: {}\r\n\r\n", path, host);
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
    String::from_utf8_lossy(&buffer).to_string()
}
