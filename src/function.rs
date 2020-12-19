use crate::bytecode::BytecodeChunk;
use crate::shared::SharedImmutable;
use crate::value_type::ValueType;

#[derive(Debug)]
pub struct Function {
    name: Option<SharedImmutable<String>>,
    parameters: Vec<SharedImmutable<String>>,
    bytecode: SharedImmutable<BytecodeChunk>,
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for Function {}

impl Function {
    pub fn new(
        name: Option<SharedImmutable<String>>,
        parameters: Vec<SharedImmutable<String>>,
        bytecode: SharedImmutable<BytecodeChunk>,
    ) -> Self {
        Function {
            name,
            parameters,
            bytecode,
        }
    }

    pub fn type_of(&self) -> ValueType {
        ValueType::Function
    }

    pub fn to_boolean(&self) -> bool {
        true
    }

    pub fn to_string(&self) -> String {
        match self.name() {
            Some(name) => format!("function:{}", *name),
            None => "<function>".into(),
        }
    }

    pub fn name(&self) -> Option<SharedImmutable<String>> {
        self.name.clone()
    }

    pub fn parameters(&self) -> &Vec<SharedImmutable<String>> {
        &self.parameters
    }

    pub fn bytecode(&self) -> &SharedImmutable<BytecodeChunk> {
        &self.bytecode
    }
}
