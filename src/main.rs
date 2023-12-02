mod core;
use core::browser_controls::Browser;
use core::interface_controls::build_interface;

// gust may need to go into interfact_controls
use gust::Gust;
use gust::layout::{VerticalLayout, HorizontalLayout, GridLayout};
use gust::widgets::{Button, Label, TextBox, Menu, MenuItem};

fn main() {
    let browser = Browser::new();
    if let Err(err) = build_interface(&browser) {
        eprintln!("Browser initialization failed: {:?}", err);
    }
}
