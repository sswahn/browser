mod network_controls;
use network_controls::http_response;
use futures::executor::block_on;
use iced::{button, Align, Application, Button, Column, Command, Container, Element, Image, Settings, Text, TextInput};

struct BrowserApp {
    entry: TextInput,
    label: Text,
    back_button: button::State,
    forward_button: button::State,
    go_button: button::State,
    add_bookmark_button: button::State,
    view_bookmarks_button: button::State,
}

enum Message {
    BackButtonPressed,
    ForwardButtonPressed,
    GoButtonPressed,
    AddBookmarkButtonPressed,
    ViewBookmarksButtonPressed,
    UrlChanged(String),
}

impl Application for BrowserApp {
    type Executor = iced::executor::Default;
    type Message = Message;

    fn new() -> (Self, Command<Self::Message>) {
        (
            Self::default(),
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("My Iced Browser")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::BackButtonPressed => {
                self.browser.back(); // Access Browser methods
            }
            Message::ForwardButtonPressed => {
                self.browser.forward(); // Access Browser methods
            }
            Message::GoButtonPressed => {
                let url = self.entry.value.clone();
                self.browser.navigate(&url); // Access Browser methods
                // Handle HTTP response and update label
            }
            Message::AddBookmarkButtonPressed => {
                add_bookmark_dialog(&self.browser); // Access Browser methods
            }
            Message::ViewBookmarksButtonPressed => {
                view_bookmarks_dialog(&self.browser); // Access Browser methods
            }
            Message::UrlChanged(new_url) => {
                self.entry.value = new_url;
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        // Same as before
    }
}
