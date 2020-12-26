#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    pub index: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub path: Option<String>,
    pub start: Position,
    pub end: Position,
}
