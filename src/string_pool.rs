#[derive(Debug, Clone)]
pub struct StringPool {
    strings: Vec<String>,
}

impl StringPool {
    pub fn new() -> Self {
        Self {
            strings: Vec::with_capacity(100),
        }
    }

    pub fn add_string(&mut self, s: &str) -> usize {
        let maybe_position = self.strings.iter().position(|pool_string| pool_string == s);
        if let Some(position) = maybe_position {
            return position;
        }
        self.strings.push(s.to_owned());
        self.strings.len() - 1
    }

    pub fn get_string_by_idx(&self, idx: usize) -> Option<String> {
        if let Some(s) = self.strings.get(idx) {
            return Some(s.to_owned());
        }
        None
    }
}

pub fn initialize_new_string_pool() -> StringPool {
    let mut map = StringPool::new();
    // add keywords
    map.add_string("var");
    map.add_string("let");
    map.add_string("const");
    map.add_string("function");
    map.add_string("return");
    map.add_string("if");
    map.add_string("else");
    return map;
}
