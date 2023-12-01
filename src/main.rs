mod core;
use core::browser_controls::Browser;
use core::interface_controls::build_interface;
use gust::{Gust, Button, TextBox, Label, Styles};

fn main() {
    let browser = Browser::new();
    if let Err(err) = build_interface(&browser) {
        eprintln!("Browser initialization failed: {:?}", err);
    }
}
