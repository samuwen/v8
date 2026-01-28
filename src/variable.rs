use crate::{
    errors::JSError,
    values::{JSResult, JSValue},
};

pub type VariableId = usize;

#[derive(Debug)]
pub struct Variable {
    is_initialized: bool,
    is_expired: bool,
    is_mutable: bool,
    value: JSValue,
}

impl Variable {
    pub fn new(mutable: bool, value: JSValue) -> Self {
        Self {
            is_initialized: true,
            is_expired: false,
            is_mutable: mutable,
            value,
        }
    }

    pub fn expire_variable(&mut self) {
        self.is_expired = true;
    }

    pub fn is_expired(&self) -> bool {
        self.is_expired
    }

    pub fn update_value(&mut self, value: JSValue) -> JSResult<JSValue> {
        if self.is_mutable {
            self.value = value;
            return Ok(JSValue::Undefined);
        }
        Err(JSError::new_const_type_error())
    }

    pub fn get_value(&self) -> JSValue {
        self.value.clone()
    }

    pub fn is_mutable(&self) -> bool {
        self.is_mutable
    }
}
