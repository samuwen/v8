use crate::heap::HeapId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Normal,
    Break,
    Continue,
    Return(HeapId),
}

impl Default for ErrorKind {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Clone, Debug)]
pub struct JSError {
    pub kind: ErrorKind,
    pub message: String,
}

impl JSError {
    pub fn new(message: &str) -> Self {
        Self {
            kind: Default::default(),
            message: message.to_string(),
        }
    }

    pub fn new_function_type_error(name: &str) -> Self {
        Self {
            kind: Default::default(),
            message: format!("Uncaught TypeError: {} is not a function", name),
        }
    }

    pub fn new_const_type_error() -> Self {
        Self {
            kind: Default::default(),
            message: "Uncaught TypeError: Assignment to constant variable.".to_string(),
        }
    }

    pub fn new_break() -> Self {
        Self {
            kind: ErrorKind::Break,
            message: String::new(),
        }
    }

    pub fn new_continue() -> Self {
        Self {
            kind: ErrorKind::Continue,
            message: String::new(),
        }
    }

    pub fn new_return(id: HeapId) -> Self {
        Self {
            kind: ErrorKind::Return(id),
            message: String::new(),
        }
    }

    pub fn new_not_found(kind: &str, id: usize) -> Self {
        Self {
            kind: ErrorKind::Normal,
            message: format!("{kind} with id {id} not found"),
        }
    }
}
