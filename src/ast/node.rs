use uuid::Uuid;

use super::parser::ParsePair;

#[derive(Debug)]
pub struct AstNodeInfo {
    pub id: Uuid,
    pub start: usize,
    pub end: usize,
}

impl AstNodeInfo {
    pub fn new(pair: &ParsePair) -> Self {
        Self {
            id: Uuid::new_v4(),
            start: pair.as_span().start(),
            end: pair.as_span().end(),
        }
    }
}
