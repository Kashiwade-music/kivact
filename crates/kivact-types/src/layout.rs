//! Role: Layout keywords: flex direction, alignment, position, overflow, placement.

/// Main-axis direction and ordering of children in a flex container.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum FlexDirection {
    Row,
    #[default]
    Column,
    RowReverse,
    ColumnReverse,
}

/// Cross-axis alignment of a box or its children.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
}

/// Side of the anchor an overlay asks for. The engine flips it to the
/// opposite side when it does not fit and clamps it into the window
/// according to the host's placement policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Placement {
    Below,
    Above,
    Start,
    End,
    Center,
}
