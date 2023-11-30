use std::collections::VecDeque;
use std::collections::HashMap;

struct Browser {
    history: VecDeque<String>,
    current_url: Option<String>,
    cache: HashMap<String, String>,
    bookmarks: HashMap<String, String>,
}

impl Browser {
    pub fn new() -> Self {
        Browser {
            history: VecDeque::new(),
            current_url: None,
            cache: HashMap::new(),
            bookmarks: HashMap::new(),
        }
    }

    pub fn navigate(&mut self, url: String) {
        self.history.push_back(url.to_string());
        self.current_url = Some(url.to_string());
    }

    pub fn back(&mut self) -> Option<String> {
        self.history.pop_back().map(|url| {
            self.current_url = self.history.back().cloned();
            url
        })
    }

    pub fn forward(&mut self) -> Option<String> {
        self.history.pop_front().map(|url| {
            self.current_url = self.history.front().cloned();
            url
        })
    }

    pub fn set_cache(&mut self, url: &str, response: String) {
        self.cache.insert(url.to_string(), response);
    }

    pub fn get_cache(&self, url: &str) -> Option<&String> {
        self.cache.get(url)
    }

    pub fn remove_from_cache(&mut self, url: &str) {
        self.cache.remove(url);
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn add_bookmark(&mut self, url: &str, title: &str) {
        self.bookmarks.insert(title.to_string(), url.to_string());
    }

    pub fn get_bookmarks(&self) -> &HashMap<String, String> {
        &self.bookmarks
    }
}
