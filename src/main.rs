mod core;
use core::browser_controls::Browser;

use native_tls::{TlsConnector, TlsStream};
use std::net::TcpStream;
use std::str;
use std::sync::Mutex;
use gtk::prelude::*;
use gtk::{Label, Button, Entry, Window, WindowType};
use tokio::task;

const HTTP_PORT: u16 = 80;
const HTTPS_PORT: u16 = 443;
const HTTPS_PREFIX: &str = "https://";
const HTTP_PREFIX: &str = "http://";

fn main() {
    let browser_mutex = Mutex::new(Browser::new());
    build_gui(&browser_mutex)
}

fn build_gui(browser: &Mutex<Browser>) {
    gtk::init().expect("Failed to initialize GTK.");
    let window = Window::new(WindowType::Toplevel); 
    let entry = Entry::new();
    let button = Button::new_with_label("Go");
    let label = Label::new(None);
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    vbox.add(&entry);
    vbox.add(&button);
    vbox.add(&label);
    window.add(&vbox);

    button.connect_clicked(move |_| {
        handle_button_click(&entry, &label, &browser);
    });

    // Handle window close event.
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    window.show_all(); // Show all UI elements.
    gtk::main(); // Start the GTK main loop.
}

fn handle_button_click(entry: &Entry, label: &Label, browser: &Mutex<Browser>) {
    let url = entry.get_text().unwrap_or_else(|| String::from(""));
    let mut browser = browser.lock().unwrap();
    browser.navigate(&url);
    if let Some(cached_response) = browser.get_cache(&url) {
        label.set_text(cached_response);
        return;
    }
    
    label.set_text("Loading...");
    
    let (host, path) = parse_url(&url);
    let port = get_port(&url);

    task::spawn(async move {
        let stream = connect_to_stream(&host, port).await;
        let response = make_request(&stream, &host).await;

        // Update the UI on the main thread
        gtk::idle_add(move || {
            label.set_text(&response.body);
            browser.set_cache(&url, response.body);

            // stops the idle handler
            glib::Continue(false)
        });
    });
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

async fn upgrade_to_https(host: &str, stream: &mut TcpStream) -> Result<TlsStream<TcpStream>, Box<dyn std::error::Error>> {
    let connector = TlsConnector::new()?;
    let tls_stream = connector.connect(host, stream)?;
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

fn validate_url(host: &str, path: &str) -> Result<(), &'static str> {
    if host.is_empty() || path.is_empty() {
        return Err(format!("Invalid URL format: host={}, path={}", host, path));
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
