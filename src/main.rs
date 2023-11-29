mod core;
use core::browser_controls::Browser;
use core::network_controls::http_response;
use core::interface_controls::build_interface;

use std::sync::Mutex;

fn main() {
    let browser_mutex = Mutex::new(Browser::new());
    let network_response = http_response(&browser_mutex)
    if let Err(err) = build_interface(&browser_mutex, &network_response) {
        eprintln!("Browser initialization failed: {:?}", err);
    }
}
