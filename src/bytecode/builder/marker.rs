#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Marker {
    LoopStart,
    LoopEnd,
    Break,
    Continue,
}
