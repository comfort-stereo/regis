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
    Capture { offset: usize },
}

// impl Variable {
//     pub fn new(name: SharedImmutable<String>, variant: VariableVariant) -> Self {
//         Self { name, variant }
//     }

//     pub fn name(&self) -> &SharedImmutable<String> {
//         &self.name
//     }

//     pub fn variant(&self) -> &VariableVariant {
//         &self.variant
//     }
// }
