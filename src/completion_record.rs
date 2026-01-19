use crate::values::JSValue;

/// https://262.ecma-international.org/15.0/index.html#sec-completion-record-specification-type
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompletionRecordType {
    Normal,
    Throw,
    Break,
    Continue,
    Return,
}

#[derive(Clone, Debug)]
pub struct CompletionRecord {
    kind: CompletionRecordType,
    target: Option<String>,
    value: Option<JSValue>,
}

impl CompletionRecord {
    pub fn complete_normal(value: JSValue) -> CompletionRecord {
        CompletionRecord {
            kind: CompletionRecordType::Normal,
            target: None,
            value: Some(value),
        }
    }

    pub fn complete_throw(value: JSValue) -> CompletionRecord {
        CompletionRecord {
            kind: CompletionRecordType::Throw,
            target: None,
            value: Some(value),
        }
    }

    pub fn complete_break(target: String) -> CompletionRecord {
        CompletionRecord {
            kind: CompletionRecordType::Break,
            target: Some(target),
            value: None,
        }
    }

    pub fn complete_return(target: String, value: JSValue) -> CompletionRecord {
        CompletionRecord {
            kind: CompletionRecordType::Return,
            target: Some(target),
            value: Some(value),
        }
    }

    pub fn complete_continue(target: String) -> CompletionRecord {
        CompletionRecord {
            kind: CompletionRecordType::Continue,
            target: Some(target),
            value: None,
        }
    }

    pub fn update_empty(&self, value: JSValue) -> CompletionRecord {
        if self.value.is_some() {
            return self.clone();
        }
        CompletionRecord {
            kind: self.kind.clone(),
            target: self.target.clone(),
            value: Some(value),
        }
    }

    pub fn is_normal(&self) -> bool {
        self.kind == CompletionRecordType::Normal
    }

    pub fn is_throw(&self) -> bool {
        self.kind == CompletionRecordType::Throw
    }

    pub fn is_return(&self) -> bool {
        self.kind == CompletionRecordType::Return
    }

    pub fn is_continue(&self) -> bool {
        self.kind == CompletionRecordType::Continue
    }

    pub fn is_break(&self) -> bool {
        self.kind == CompletionRecordType::Break
    }

    pub fn get_value(&self) -> &Option<JSValue> {
        &self.value
    }
}
