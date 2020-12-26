use uuid::Uuid;

use super::grammar::GrammarPair;

#[derive(Debug)]
pub struct AstNodeInfo {
    pub id: Uuid,
    pub start: usize,
    pub end: usize,
}

impl AstNodeInfo {
    pub fn new(pair: &GrammarPair) -> Self {
        Self {
            id: Uuid::new_v4(),
            start: pair.as_span().start(),
            end: pair.as_span().end(),
        }
    }
}
