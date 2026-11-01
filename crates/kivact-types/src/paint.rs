//! Role: How a gradient continues beyond its first and last stop.

/// How a gradient continues beyond its first and last stop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Extend {
    /// The end colors continue.
    #[default]
    Pad,
    /// The stops repeat (CSS `repeating-*-gradient`).
    Repeat,
    /// The stops repeat, every other time backwards.
    Reflect,
}
