mod core;
use core::browser_controls::Browser;
use core::interface_controls::build_interface;

use native_tls::{TlsConnector, TlsStream};
use std::net::TcpStream;
use std::str;
use std::sync::Mutex;
use gtk::prelude::*;
use gtk::{Button, Entry, Image, Label, Menu, MenuBar, MenuItem, Window, WindowType};
use tokio::task;

const HTTP_PORT: u16 = 80;
const HTTPS_PORT: u16 = 443;
const HTTPS_PREFIX: &str = "https://";
const HTTP_PREFIX: &str = "http://";

enum BrowserError {
    InvalidUrlFormat { host: String, path: String },
    TlsError(Box<dyn std::error::Error>),
}

fn main() {
    let browser_mutex = Mutex::new(Browser::new());
    if let Err(err) = build_interface(&browser_mutex) {
        eprintln!("Browser initialization failed: {:?}", err);
    }
}
