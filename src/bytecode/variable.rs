use crate::path::CanonicalPath;
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
    Capture { location: StackLocation },
}

pub enum VariableLocation {
    Stack(StackLocation),
    Export(ExportLocation),
    Global(GlobalLocation),
}

#[derive(Debug, Clone)]
pub struct StackLocation {
    pub ascend: usize,
    pub address: usize,
}

#[derive(Debug, Clone)]
pub struct ExportLocation {
    pub path: CanonicalPath,
    pub export: SharedImmutable<String>,
}

#[derive(Debug, Clone)]
pub struct GlobalLocation {
    pub address: usize,
}
