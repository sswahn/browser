use gtk::prelude::*;
use gtk::{Box, Button, Dialog, Entry, Image, Label, Menu, MenuBar, MenuItem, ResponseType, Window, WindowType};
use tokio::task;

enum BrowserError {
    IoError(std::io::Error),
}

fn build_browser(browser: &Mutex<Browser>) -> Result<(), BrowserError> {
    gtk::init().map_err(|e| BrowserError::IoError(e))?;
    let window = Window::new(WindowType::Toplevel); 
    let entry = Entry::new();
    let label = Label::new(None);
    let (back_button, forward_button, go_button) = build_navigation_buttons(&entry, &label, &browser);
    let bookmarks_menu_bar = build_bookmarks_menu(&browser);
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);

    // Handle window close event.
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    vbox.add(&entry);
    vbox.add(&back_button);
    vbox.add(&forward_button);
    vbox.add(&go_button);
    vbox.add(&label);
    vbox.add(&bookmarks_menu_bar);
    window.add(&vbox);
    window.show_all(); // Show all UI elements.
    gtk::main(); // Start the GTK main loop.
    Ok(())
}

fn build_navigation_buttons(entry: &Entry, label: &Label, browser: &Mutex<Browser>) -> (Button, Button, Button) {
    let back_icon = Image::from_icon_name(Some("go-back"), IconSize::Button.into());
    let forward_icon = Image::from_icon_name(Some("go-forward"), IconSize::Button.into());
    let go_icon = Image::from_icon_name(Some("gtk-ok"), IconSize::Button.into());
    let back_button = Button::new_with_label("Back").set_image(Some(&back_icon));
    let forward_button = Button::new_with_label("Forward").set_image(Some(&forward_icon));
    let go_button = Button::new_with_label("Go").set_image(Some(&go_icon));
    go_button.connect_clicked(move |_| {
        handle_go_button_click(&entry, &label, &browser);
    });
    back_button.connect_clicked(move |_| {
        handle_back_button_click(&browser);
    });
    forward_button.connect_clicked(move |_| {
        handle_forward_button_click(&browser);
    });
    (back_button, forward_button, go_button)
}

fn build_bookmarks_menu(browser: &Mutex<Browser>) -> Menu {
    let bookmarks_menu_bar = Menu::new();
    let bookmarks_menu_button = MenuItem::new_with_label("Bookmarks");
    let bookmarks_menu = Menu::new();
    let add_bookmark_item = MenuItem::new_with_label("Add Bookmark");
    let view_bookmarks_item = MenuItem::new_with_label("View Bookmarks");
    bookmarks_menu.append(&add_bookmark_item);
    bookmarks_menu.append(&view_bookmarks_item);
    bookmarks_menu_button.set_submenu(Some(&bookmarks_menu));
    bookmarks_menu_bar.append(&bookmarks_menu_button);
    let browser_clone = browser.clone();
    add_bookmark_item.connect_activate(move |_| {
        add_bookmark_dialog(&browser_clone);
    });
    view_bookmarks_item.connect_activate(move |_| {
        view_bookmarks_dialog(&browser_clone);
    });
    bookmarks_menu_bar
}

fn add_bookmark_dialog(browser: &Mutex<Browser>) {
    let dialog = Dialog::new();
    dialog.set_title("Add Bookmark");

    let title_label = Label::new(Some("Bookmark Title:"));
    let title_entry = Entry::new();

    let url_label = Label::new(Some("Bookmark URL:"));
    let url_entry = Entry::new();

    let add_button = Button::new_with_label("Add");
    let cancel_button = Button::new_with_label("Cancel");

    let content_area = dialog.get_content_area();
    content_area.add(&title_label);
    content_area.add(&title_entry);
    content_area.add(&url_label);
    content_area.add(&url_entry);
    content_area.add(&add_button);
    content_area.add(&cancel_button);

    add_button.connect_clicked(move |_| {
        let title = title_entry.get_text().unwrap_or_else(|| String::from(""));
        let url = url_entry.get_text().unwrap_or_else(|| String::from(""));
        let mut browser = browser.lock().unwrap();
        browser.add_bookmark(&url, &title);
        dialog.close();
    });

    cancel_button.connect_clicked(|_| {
        dialog.close();
    });

    dialog.show_all();
}

fn view_bookmarks_dialog(browser: &Mutex<Browser>) {
    let dialog = Dialog::new();
    dialog.set_title("Bookmarks");

    let bookmarks_label = Label::new(Some("Bookmarks:"));

    let bookmarks_text = browser.lock().unwrap().get_bookmarks().iter()
        .map(|(title, url)| format!("{}: {}", title, url))
        .collect::<Vec<String>>()
        .join("\n");

    let bookmarks_entry = Entry::new();
    bookmarks_entry.set_text(&bookmarks_text);
    bookmarks_entry.set_editable(false);

    let close_button = Button::new_with_label("Close");

    let content_area = dialog.get_content_area();
    content_area.add(&bookmarks_label);
    content_area.add(&bookmarks_entry);
    content_area.add(&close_button);

    close_button.connect_clicked(|_| {
        dialog.close();
    });

    dialog.show_all();
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
    
    let (host, path) = parse_url(&url);
    let port = get_port(&url);

    // Update the UI on the main thread
    gtk::idle_add(move || {
        label.set_text(&response.body); // response needs to be passed in main.rs to parent in this mod
        browser.set_cache(&url, &response.body);

        // stops the idle handler
        glib::Continue(false)
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
