//! Role: Layout keywords: flex, alignment, position, overflow, placement, scroll axes.

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

/// Side of the anchor an overlay asks for. The engine flips it to the
/// opposite side when it does not fit and clamps it into the window
/// according to the host's placement policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    Below,
    Above,
    Start,
    End,
    Center,
}

/// Where an overlay sits along its anchor's side ([`Placement`]): its start,
/// centre or end (`Below` with `End`: right edges aligned). Not the flex
/// [`Align`] of a container's children.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum OverlayAlign {
    /// At the anchor's start edge.
    #[default]
    Start,
    /// Centred on the anchor.
    Center,
    /// At the anchor's end edge.
    End,
}

/// The axes a scroll container scrolls along. An axis that does not scroll
/// clips.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ScrollAxes {
    /// Up and down (the default); the content is laid out in a column.
    #[default]
    Vertical,
    /// Sideways; the content is laid out in a row.
    Horizontal,
    /// Both ways (a table or a code block wider and taller than the
    /// container); the content is laid out in a column.
    Both,
}
