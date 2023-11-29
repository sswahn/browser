struct Browser {
    history: VecDeque<String>,
    current_url: Option<String>,
    cache: HashMap<String, String>,
}

impl Browser {
    fn new() -> Self {
        Browser {
            history: VecDeque::new(),
            current_url: None,
            cache: HashMap::new(),
        }
    }

    fn navigate(&mut self, url: String) {
        self.history.push_back(url.clone());
        self.current_url = url;
    }

    fn back(&mut self) -> Option<String> {
        self.history.pop_back().map(|url| {
            self.current_url = self.history.back().cloned();
            url
        })
    }

    fn forward(&mut self) -> Option<String> {
        self.history.pop_front().map(|url| {
            self.current_url = self.history.front().cloned();
            url
        })
    }

    fn refresh(&mut self) {
        let url = self.current_url.clone()
        self.cache.remove(&url);
    }

    fn set_cache(&mut self, url: &str, response: String) {
        self.cache.insert(url.to_string(), response);
    }

    fn get_cache(&self, url: &str) -> Option<&String> {
        self.cache.get(url)
    }
}
