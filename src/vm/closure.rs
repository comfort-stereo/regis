use super::Value;

#[derive(Debug, Clone)]
pub struct Capture {
    pub value: Value,
}

#[derive(Debug)]
pub struct Closure {
    captures: Vec<Capture>,
}

impl Default for Closure {
    fn default() -> Self {
        Self::new()
    }
}

impl Closure {
    pub fn new() -> Self {
        Self {
            captures: Vec::new(),
        }
    }
}
