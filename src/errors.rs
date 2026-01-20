use string_interner::symbol::SymbolU32;

use crate::global::get_string_from_pool;

#[derive(Clone, Debug)]
pub struct JSError {
    pub message: String,
}

impl JSError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    pub fn new_type_error(name: &str) -> Self {
        Self {
            message: format!("Uncaught TypeError: {} is not a function", name),
        }
    }

    pub fn new_declaration_error(name: &str) -> Self {
        Self {
            message: format!("Variable declaration expected."),
        }
    }

    pub fn new_type_error_sym(name: &SymbolU32) -> Self {
        let string = get_string_from_pool(name).unwrap_or("unknown".to_string());
        Self::new_type_error(&string)
    }
}
