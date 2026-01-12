use crate::string_pool::{StringPool, initialize_new_string_pool};

#[derive(Debug, Clone)]
pub struct SharedContext {
    pub string_pool: StringPool,
}

impl SharedContext {
    pub fn new() -> Self {
        Self {
            string_pool: initialize_new_string_pool(),
        }
    }
}
