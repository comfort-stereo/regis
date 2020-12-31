use super::Value;

#[derive(Debug, Clone)]
pub struct Capture {
    value: Value,
}

impl Capture {
    pub fn new(value: Value) -> Self {
        Self { value }
    }

    pub fn get(&self) -> &Value {
        &self.value
    }

    pub fn set(&mut self, value: Value) {
        self.value = value;
    }
}
