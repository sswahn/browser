mod core;
use core::browser_controls::Browser;
use core::interface_controls::build_interface;

use std::sync::Mutex;

fn main() {
    let browser_mutex = Mutex::new(Browser::new());
    if let Err(err) = build_interface(&browser_mutex) {
        eprintln!("Browser initialization failed: {:?}", err);
    }
}
