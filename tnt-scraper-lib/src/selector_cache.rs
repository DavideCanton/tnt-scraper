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

    pub fn add_and_get_selector<S: Into<String>>(&self, s: S) -> Selector {
        let owned = s.into();

        self.cache
            .borrow_mut()
            .entry(owned.clone())
            .or_insert_with(|| Selector::parse(&owned).unwrap())
            .clone()
    }
}
