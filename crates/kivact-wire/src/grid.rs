//! Role: Grid layout protocol representation.
//!
//! Grid layout (CSS Grid) of a node: as a container, its tracks; as an
//! item, its place. Boxed in [`LayoutProps::grid`](super::LayoutProps):
//! most nodes have none.

use super::values::{Align, Dimension};

/// A node's grid props.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct GridProps {
    /// It lays its children out in a grid with these tracks (`None`: it is
    /// a flex container).
    pub container: Option<GridContainer>,
    /// Its columns in its parent's grid (CSS `grid-column`).
    pub column: GridLine,
    /// Its rows in its parent's grid (CSS `grid-row`).
    pub row: GridLine,
    /// Its alignment along the row axis in its grid area (CSS
    /// `justify-self`; `None`: the container's `justify_items`).
    pub justify_self: Option<Align>,
}

/// The tracks and placement rules of a grid container.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct GridContainer {
    /// CSS `grid-template-columns`.
    pub columns: Vec<GridTrack>,
    /// CSS `grid-template-rows`.
    pub rows: Vec<GridTrack>,
    /// CSS `grid-auto-columns` (empty: `auto`).
    pub auto_columns: Vec<TrackSize>,
    /// CSS `grid-auto-rows` (empty: `auto`).
    pub auto_rows: Vec<TrackSize>,
    /// CSS `grid-auto-flow`.
    pub flow: GridFlow,
    /// CSS `justify-items` (`None`: stretch).
    pub justify_items: Option<Align>,
}

/// One entry of a track list: a track, or tracks repeated.
#[derive(Clone, Debug, PartialEq)]
pub enum GridTrack {
    Single(TrackSize),
    /// CSS `repeat(count, tracks)`.
    Repeat(Repeat, Vec<TrackSize>),
}

/// How often a `repeat()` repeats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Repeat {
    Count(u16),
    /// As many as fit (CSS `auto-fill`).
    AutoFill,
    /// As many as fit, empty ones collapsed (CSS `auto-fit`).
    AutoFit,
}

/// The size of one track: CSS `minmax(min, max)` (a single size is both).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackSize {
    /// Minimum track breadth.
    pub min: Breadth,
    /// Maximum track breadth.
    pub max: Breadth,
}

/// One bound of a track's size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Breadth {
    /// px, a percentage of the grid's content box, calc, `auto`,
    /// `min-content`, `max-content` (`fit-content` is `FitContent` of
    /// 100%).
    Size(Dimension),
    /// A share of the free space (CSS `fr`; as a minimum, `auto`).
    Fr(f32),
    /// CSS `fit-content(limit)` (as a minimum, `auto`).
    FitContent(Dimension),
}

/// CSS `grid-auto-flow`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GridFlow {
    #[default]
    Row,
    Column,
    RowDense,
    ColumnDense,
}

/// Where an item starts and ends along one axis (CSS `grid-column` and
/// `grid-row`: `start / end`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct GridLine {
    pub start: GridPos,
    pub end: GridPos,
}

/// One end of an item's place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GridPos {
    /// Placed by the auto-placement.
    #[default]
    Auto,
    /// At a line: from 1 at the start, from -1 at the end (CSS line
    /// numbers; 0 is `Auto`).
    Line(i16),
    /// Spanning this many tracks (CSS `span n`).
    Span(u16),
}
