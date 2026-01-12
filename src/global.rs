use crate::string_map::{StringMap, initialize_new_string_map};

#[derive(Debug, Clone)]
pub struct SharedContext {
    string_map: StringMap,
}

impl SharedContext {
    pub fn new() -> Self {
        Self {
            string_map: initialize_new_string_map(),
        }
    }

    pub fn add_string_to_map(&mut self, string: &str) -> usize {
        self.string_map.add_string(string)
    }

    pub fn get_string(&mut self, string: &str) -> Option<&usize> {
        self.string_map.get_string_index(string)
    }

    pub fn is_known_string(&self, s: &str) -> bool {
        self.string_map.is_string_in_map(s)
    }

    pub fn get_string_at_index(&self, idx: usize) -> Option<String> {
        self.string_map.get_string_by_idx(idx)
    }
}
