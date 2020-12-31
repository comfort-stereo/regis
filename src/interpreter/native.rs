use crate::error::RegisError;
use crate::shared::SharedImmutable;

use super::value::Value;
use super::Interpreter;

pub type ExternalProcedureCallback =
    fn(arguments: &[Value], context: &mut ExternalCallContext) -> Result<Value, RegisError>;

pub struct ExternalCallContext<'interpreter> {
    pub interpreter: &'interpreter mut Interpreter,
}

pub struct ExternalProcedure {
    name: SharedImmutable<String>,
    arity: usize,
    callback: ExternalProcedureCallback,
}

impl ExternalProcedure {
    pub fn new(
        name: SharedImmutable<String>,
        arity: usize,
        callback: ExternalProcedureCallback,
    ) -> Self {
        Self {
            name,
            arity,
            callback,
        }
    }

    pub fn name(&self) -> &SharedImmutable<String> {
        &self.name
    }

    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn call(
        &self,
        arguments: &[Value],
        context: &mut ExternalCallContext,
    ) -> Result<Value, RegisError> {
        (self.callback)(arguments, context)
    }
}
