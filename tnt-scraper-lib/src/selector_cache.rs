use scraper::Selector;
use std::{cell::RefCell, collections::HashMap};

pub struct SelectorCache {
    cache: RefCell<HashMap<String, Selector>>,
}

impl SelectorCache {
    pub fn new() -> SelectorCache {
        SelectorCache {
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn add_and_get_selector(&self, s: &str) -> Selector {
        let o = s.to_owned();

        self.cache
            .borrow_mut()
            .entry(o)
            .or_insert_with(|| Selector::parse(s).unwrap())
            .clone()
    }
}
