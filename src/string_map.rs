use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct StringMap {
    counter: usize,
    strings: HashMap<String, usize>,
}

impl StringMap {
    pub fn new() -> Self {
        Self {
            counter: 0,
            strings: HashMap::new(),
        }
    }

    pub fn add_string(&mut self, s: &str) -> usize {
        if let Some(number) = self.get_string_index(s) {
            return *number;
        }
        let current_counter = self.counter;
        self.strings.insert(s.to_owned(), current_counter);
        self.counter += 1;
        current_counter
    }

    pub fn get_string_index(&mut self, s: &str) -> Option<&usize> {
        self.strings.get(s)
    }

    pub fn is_string_in_map(&self, s: &str) -> bool {
        self.strings.contains_key(s)
    }

    pub fn get_string_by_idx(&self, idx: usize) -> Option<String> {
        let keys = self.strings.keys().collect::<Vec<&String>>();
        if let Some(s) = keys.get(idx) {
            return Some(s.to_string());
        }
        None
    }
}

pub fn initialize_new_string_map() -> StringMap {
    let mut map = StringMap::new();
    // add keywords
    map.add_string("let");
    map.add_string("const");
    map.add_string("function");
    map.add_string("return");
    map.add_string("if");
    map.add_string("else");
    return map;
}
