use crate::ast::location::Location;

use super::value::ValueType;

#[derive(Debug)]
pub struct VmError {
    pub location: Option<Location>,
    pub variant: VmErrorVariant,
}

impl VmError {
    pub fn new(location: Option<Location>, variant: VmErrorVariant) -> Self {
        Self { location, variant }
    }
}

#[derive(Debug)]
pub enum VmErrorVariant {
    UndefinedBinaryOperation {
        operation: String,
        target_type: ValueType,
        other_type: ValueType,
    },
    UndefinedUnaryOperation {
        operation: String,
        target_type: ValueType,
    },
    InvalidIndexAccess {
        target_type: ValueType,
        index: String,
    },
    InvalidIndexAssignment {
        target_type: ValueType,
        index: String,
    },
}
