//! Role: Defines layout lengths, edge sets and the straight-alpha color.

/// Straight-alpha color, components in `0.0..=1.0`.
pub use kivact_types::Rgba;

/// A layout length. Percentages (and a calc's) are of the containing
/// block's size along the axis (padding, margins: its width), like CSS.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Dimension {
    #[default]
    Auto,
    Px(f32),
    Percent(f32),
    /// `percent`% of the reference plus `px` (CSS `calc(p% + npx)`).
    Calc {
        percent: f32,
        px: f32,
    },
    /// CSS `min-content`, `max-content` and `fit-content`: a size (width,
    /// height, flex basis, a grid track) from the content. Elsewhere they
    /// are `Auto`.
    MinContent,
    MaxContent,
    FitContent,
}

/// Values for the physical top, right, bottom and left edges.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Edges<T> {
    /// Top edge.
    pub top: T,
    /// Right edge.
    pub right: T,
    /// Bottom edge.
    pub bottom: T,
    /// Left edge.
    pub left: T,
}

impl<T: Copy> Edges<T> {
    /// Use the same value on every edge.
    pub fn all(v: T) -> Self {
        Edges {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }
}
