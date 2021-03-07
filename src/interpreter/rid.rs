#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rid(u128);

impl Rid {
    pub fn new() -> Rid {
        Rid(0)
    }

    pub fn next(&self) -> Rid {
        Rid(self.0.wrapping_add(1))
    }
}

impl Default for Rid {
    fn default() -> Self {
        Rid::new()
    }
}
