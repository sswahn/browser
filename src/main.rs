mod core;
use core::browser_controls::Browser;
use core::interface_controls::build_interface;

fn main() {
    let browser_mutex = Mutex::new(Browser::new());
    let stream = connect_to_stream(&host, port).await;  // needs stream
    let response = make_request(&stream, &host).await;  // needs response from request

    if let Err(err) = build_interface(&browser_mutex) {
        eprintln!("Browser initialization failed: {:?}", err);
    }
}


fn handle_go_button_click(entry: &Entry, label: &Label, browser: &Mutex<Browser>) {
    let url = entry.get_text().unwrap_or(String::from(""));
    let mut browser = browser.lock().unwrap();
    browser.navigate(&url);
    if let Some(cached_response) = browser.get_cache(&url) {
        label.set_text(cached_response);
        return;
    }
    
    label.set_text("Loading...");
    
    let (host, path) = parse_url(&url); // needs to parse url
    let port = get_port(&url);          // needs port

    task::spawn(async move {
        let stream = connect_to_stream(&host, port).await;  // needs stream
        let response = make_request(&stream, &host).await;  // needs response from request

        // Update the UI on the main thread
        gtk::idle_add(move || {
            label.set_text(&response.body);
            browser.set_cache(&url, &response.body);

            // stops the idle handler
            glib::Continue(false)
        });
    });
}
