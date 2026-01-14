use string_interner::{StringInterner, backend::StringBackend, symbol::SymbolU32};

#[derive(Debug, Clone)]
pub struct SharedContext {
    pub string_pool: StringInterner<StringBackend>,
}

impl SharedContext {
    pub fn new() -> Self {
        let mut pool = StringInterner::new();
        pool.get_or_intern("const");
        pool.get_or_intern("return");
        pool.get_or_intern("true");
        pool.get_or_intern("false");
        pool.get_or_intern("undefined");
        pool.get_or_intern("for");
        pool.get_or_intern("function");
        pool.get_or_intern("let");
        pool.get_or_intern("if");
        pool.get_or_intern("else");
        pool.get_or_intern("break");
        pool.get_or_intern("continue");
        pool.get_or_intern("while");
        pool.get_or_intern("null");
        pool.get_or_intern("var");

        Self { string_pool: pool }
    }

    pub fn get_string_from_pool(&self, symbol: SymbolU32) -> String {
        if let Some(string) = self.string_pool.resolve(symbol) {
            return string.to_string();
        }
        panic!("Uninitialized string found"); // TODO flesh out error handling
    }
}
