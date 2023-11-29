mod core;
use core::browser_controls::Browser;

use native_tls::{TlsConnector, TlsStream};
use std::net::TcpStream;
use std::str;
use std::sync::Mutex;
use gtk::prelude::*;
use gtk::{Button, Entry, Image, Label, Window, WindowType};
use tokio::task;

const HTTP_PORT: u16 = 80;
const HTTPS_PORT: u16 = 443;
const HTTPS_PREFIX: &str = "https://";
const HTTP_PREFIX: &str = "http://";

enum BrowserError {
    InvalidUrlFormat { host: String, path: String },
    TlsError(Box<dyn std::error::Error>),
    IoError(std::io::Error),
    // Add more error variants as needed
}

fn main() {
    let browser_mutex = Mutex::new(Browser::new());
    if let Err(err) = build_browser(&browser_mutex) {
        eprintln!("Browser initialization failed: {:?}", err);
    }
}

fn build_browser(browser: &Mutex<Browser>) -> Result<(), BrowserError> {
    gtk::init().map_err(|e| BrowserError::IoError(e))?;
    let window = Window::new(WindowType::Toplevel); 
    let entry = Entry::new();

    let go_icon = Image::from_icon_name(Some("gtk-ok"), IconSize::Button.into());
    let back_icon = Image::from_icon_name(Some("go-back"), IconSize::Button.into());
    let forward_icon = Image::from_icon_name(Some("go-forward"), IconSize::Button.into());
    
    let go_button = Button::new_with_label("Go").set_image(Some(&go_icon));
    let back_button = Button::new_with_label("Back").set_image(Some(&back_icon));
    let forward_button = Button::new_with_label("Forward").set_image(Some(&forward_icon));

    let label = Label::new(None);
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    vbox.add(&entry);
    vbox.add(&button);
    vbox.add(&label);
    window.add(&vbox);

    go_button.connect_clicked(move |_| {
        handle_go_button_click(&entry, &label, &browser);
    });
    
    back_button.connect_clicked(move |_| {
        handle_back_button_click(&browser);
    });
    
    forward_button.connect_clicked(move |_| {
        handle_forward_button_click(&browser);
    });

    // Handle window close event.
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    window.show_all(); // Show all UI elements.
    gtk::main(); // Start the GTK main loop.
    Ok(())
}

fn handle_go_button_click(entry: &Entry, label: &Label, browser: &Mutex<Browser>) {
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

fn handle_back_button_click(browser: &Mutex<Browser>) {
    let mut browser = browser.lock().unwrap();
    browser.back();
}

fn handle_forward_button_click(browser: &Mutex<Browser>) {
    let mut browser = browser.lock().unwrap();
    browser.forward();
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
