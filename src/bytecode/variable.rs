use crate::shared::SharedImmutable;

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: SharedImmutable<String>,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: SharedImmutable<String>,
    pub variant: VariableVariant,
}

#[derive(Debug, Clone)]
pub enum VariableVariant {
    Local,
    Capture { location: VariableLocation },
}

#[derive(Debug, Clone)]
pub struct VariableLocation {
    pub ascend: usize,
    pub address: usize,
}
