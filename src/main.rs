use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::str;

const HTTP_PORT: u16 = 80;

fn main() {
    let url = read_user_input("Enter URL: ");
    let (host, path) = parse_url(&url);

    // Connect to the host
    match TcpStream::connect(format!("{}:{}", host, HTTP_PORT)) {
        Ok(mut stream) => {
            // Send a simple HTTP GET request
            let request = format!("GET {} HTTP/1.0\r\nHost: {}\r\n\r\n", path, host);
            write_to_stream(&mut stream, &request);

            // Read and print the response
            let response = read_from_stream(&mut stream);
            println!("{}", response);
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

fn write_to_stream(stream: &mut TcpStream, data: &str) {
    stream.write(data.as_bytes()).expect("Failed to write to stream");
}

fn read_from_stream(stream: &mut TcpStream) -> String {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).expect("Failed to read from stream");
    String::from_utf8_lossy(&buffer).to_string()
}
