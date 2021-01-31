use super::path::CanonicalPath;
use super::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    path: Option<CanonicalPath>,
    span: Span,
}

impl Location {
    pub fn new(path: Option<CanonicalPath>, span: Span) -> Self {
        Self { path, span }
    }

    pub fn path(&self) -> &Option<CanonicalPath> {
        &self.path
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}
