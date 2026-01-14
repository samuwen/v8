use string_interner::{StringInterner, backend::StringBackend};

#[derive(Debug, Clone)]
pub struct SharedContext {
    pub string_pool: StringInterner<StringBackend>,
}

impl SharedContext {
    pub fn new() -> Self {
        Self {
            string_pool: StringInterner::new(),
        }
    }
}
