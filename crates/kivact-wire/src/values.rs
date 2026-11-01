//! Role: Defines scalar layout, color and alignment values.

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

/// Main-axis direction and ordering of children in a flex container.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FlexDirection {
    Row,
    #[default]
    Column,
    RowReverse,
    ColumnReverse,
}

/// Cross-axis alignment of a box or its children.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Align {
    Start,
    Center,
    End,
    #[default]
    Stretch,
    Baseline,
}

/// How content is distributed along an axis (CSS `justify-content`,
/// `align-content`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Justify {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    /// Auto tracks (grid) or lines (wrapping flex) grow to fill; `Start`
    /// for `justify-content` in flex.
    Stretch,
}

/// How a box is positioned (CSS `position`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Position {
    #[default]
    Relative,
    /// Out of flow, at its insets in its parent's padding box.
    Absolute,
    /// In flow, then moved by the scroll of its nearest scrolling ancestor
    /// to keep its insets from that scrollport's edges, within its
    /// parent's content box (the engine's geometry, after layout).
    Sticky,
    /// Out of flow, at its insets in the window; not scrolled or clipped
    /// by its ancestors.
    Fixed,
}

/// Whether overflowing content remains visible, is clipped or scrolls.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
}
