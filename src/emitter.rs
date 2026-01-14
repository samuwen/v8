use crate::{global::SharedContext, stmt::Stmt, value::Value};

pub struct Emitter<'a> {
    context: &'a mut SharedContext,
    statements: Vec<Stmt>,
}

impl<'a> Emitter<'a> {
    pub fn new(context: &'a mut SharedContext, statements: Vec<Stmt>) -> Self {
        Self {
            context,
            statements,
        }
    }

    pub fn evaluate(&mut self) -> Value {
        let stmt = self.statements.get(0).unwrap();
        stmt.evaluate(&mut self.context)
    }
}
