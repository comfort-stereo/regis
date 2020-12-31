use crate::path::CanonicalPath;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    pub index: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub path: Option<CanonicalPath>,
    pub start: Position,
    pub end: Position,
}
