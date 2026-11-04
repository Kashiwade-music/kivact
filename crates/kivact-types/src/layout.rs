//! Role: Layout configuration, leaf measurement and overlay constraints.
use crate::{Edges, NodeId, Point, Rect, Size};

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
    Repeat(GridRepeat, Vec<TrackSize>),
}

/// How often a `repeat()` repeats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridRepeat {
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

/// Space available to a node along one axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AvailableSpace {
    /// A definite number of pixels.
    Definite(f32),
    /// As little as the content allows.
    MinContent,
    /// As much as the content wants.
    MaxContent,
}

/// What a leaf is measured under.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeasureRequest {
    /// Width the layout already fixed.
    pub known_width: Option<f32>,
    /// Height the layout already fixed.
    pub known_height: Option<f32>,
    /// Width available.
    pub available_width: AvailableSpace,
    /// Height available.
    pub available_height: AvailableSpace,
}

impl MeasureRequest {
    /// A request for `width` fixed and any height (a host checks whether
    /// new content keeps its size this way, outside a layout pass).
    pub fn at_width(width: f32) -> Self {
        MeasureRequest {
            known_width: Some(width),
            known_height: None,
            available_width: AvailableSpace::Definite(width),
            available_height: AvailableSpace::MaxContent,
        }
    }
}

/// Where a measured leaf's first baseline lies, vertically in its content
/// box (inside border and padding).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Baseline {
    /// This far below the top of the content box.
    FromTop(f32),
    /// This far below the vertical center of the content box (negative:
    /// above it): a line centered in a box taller than itself.
    FromCenter(f32),
}

/// A measured leaf's content size and, when it has text on a line, its
/// first baseline.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LeafSize {
    pub size: Size,
    /// `None`: the leaf has no baseline, and a flex container aligning it
    /// by baseline takes the bottom of its box.
    pub baseline: Option<Baseline>,
}

impl From<Size> for LeafSize {
    fn from(size: Size) -> Self {
        LeafSize {
            size,
            baseline: None,
        }
    }
}

/// Soft wrapping and automatic visual row bounds of a native multiline field.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditLineLayout {
    pub wrap: bool,
    pub rows: Option<(usize, usize)>,
}

/// Where an overlay is placed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OverlayAt {
    /// On a side of the drawn box of `node`; `None` (the `NodeRef` is bound to
    /// no node, or the node is gone): centred in the window. `at_least_width`:
    /// lay the overlay out at least as wide as `node`'s layout box (its
    /// minimum width is raised to it).
    Anchor {
        node: Option<NodeId>,
        at_least_width: bool,
    },
    /// At a window point (an empty anchor rectangle there).
    Point(Point),
}

/// Viewport fitting parameters for an overlay that opts into edge insets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayFit {
    pub margin: f32,
    pub keep_anchor_clear: bool,
    pub rtl: bool,
}

impl OverlayFit {
    /// Inset each axis by as much of the requested margin as fits.
    pub fn viewport(self, size: Size) -> Rect {
        let margin = if self.margin.is_finite() {
            self.margin.max(0.0)
        } else {
            0.0
        };
        let x = margin.min(size.width / 2.0);
        let y = margin.min(size.height / 2.0);
        Rect {
            x,
            y,
            width: size.width - 2.0 * x,
            height: size.height - 2.0 * y,
        }
    }
}

/// Props of an `Overlay` node. While it is shown, a host sends
/// `HostEvent::A11y` with `A11yRequest::Dismiss(DismissReason::OutsidePress)`
/// to an overlay whose `Actions` offer `Dismiss` when a pointer press lands
/// outside its subtree and its anchor's subtree. A `modal` overlay confines
/// input to its subtree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayProps {
    /// Where it is placed.
    pub at: OverlayAt,
    /// Requested side of the anchor.
    pub placement: Placement,
    /// Alignment along the requested anchor side.
    pub align: OverlayAlign,
    /// The gap between the anchor and the overlay (logical px).
    pub offset: f32,
    /// Optional viewport bounds and ordered placement fallback.
    pub fit: Option<OverlayFit>,
    /// Confine interaction to this overlay while shown.
    pub modal: bool,
}

impl OverlayProps {
    /// The node it is anchored to, if any (a point overlay has none).
    pub fn anchor(&self) -> Option<NodeId> {
        match self.at {
            OverlayAt::Anchor { node, .. } => node,
            OverlayAt::Point(_) => None,
        }
    }
}

/// Flex or grid layout of a node. Hosts own the layout algorithm.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutProps {
    /// Main-axis direction of the flex container.
    pub direction: FlexDirection,
    /// Default cross-axis child alignment; absent uses stretch.
    pub align_items: Option<Align>,
    /// Override the parent alignment for this child.
    pub align_self: Option<Align>,
    /// CSS `justify-content` (`None`: the start in flex, `stretch` for a
    /// grid's auto tracks, like CSS `normal`).
    pub justify: Option<Justify>,
    /// CSS `align-content` (`None`: `stretch` in a grid, the start in
    /// flex).
    pub align_content: Option<Justify>,
    /// Allow children to wrap onto additional flex lines.
    pub wrap: bool,
    /// Between rows (CSS `row-gap`).
    pub row_gap: Dimension,
    /// Between columns (CSS `column-gap`).
    pub column_gap: Dimension,
    /// Space between border and content.
    pub padding: Edges<Dimension>,
    /// The border's widths in px (the paint's `Border::widths`): like
    /// padding, they take room inside the box (CSS `box-sizing:
    /// border-box`), so a change of width is a layout change and one of
    /// color or style is not.
    pub border: Edges<f32>,
    /// Space outside the border box.
    pub margin: Edges<Dimension>,
    /// Preferred border-box width.
    pub width: Dimension,
    /// Preferred border-box height.
    pub height: Dimension,
    /// Minimum border-box width.
    pub min_width: Dimension,
    /// Minimum border-box height.
    pub min_height: Dimension,
    /// Maximum border-box width.
    pub max_width: Dimension,
    /// Maximum border-box height.
    pub max_height: Dimension,
    /// Share of positive free space assigned to this flex item.
    pub grow: f32,
    /// Relative shrinking factor when flex space is insufficient.
    pub shrink: f32,
    /// Initial main-axis size of this flex item.
    pub basis: Dimension,
    /// Positioning mode relative to parent or window.
    pub position: Position,
    /// Offsets from the reference box edges.
    pub inset: Edges<Dimension>,
    /// Horizontal overflow and scrolling behavior.
    pub overflow_x: Overflow,
    /// Vertical overflow and scrolling behavior.
    pub overflow_y: Overflow,
    /// `display: none`.
    pub hidden: bool,
    /// CSS `aspect-ratio` (width / height; `None`: auto).
    pub aspect_ratio: Option<f32>,
    /// Grid container tracks and grid item placement (`None`: neither).
    pub grid: Option<Box<GridProps>>,
}

impl Default for LayoutProps {
    fn default() -> Self {
        LayoutProps {
            direction: FlexDirection::Column,
            align_items: None,
            align_self: None,
            justify: None,
            align_content: None,
            wrap: false,
            row_gap: Dimension::Px(0.0),
            column_gap: Dimension::Px(0.0),
            padding: Edges::all(Dimension::Px(0.0)),
            border: Edges::all(0.0),
            margin: Edges::all(Dimension::Px(0.0)),
            width: Dimension::Auto,
            height: Dimension::Auto,
            min_width: Dimension::Auto,
            min_height: Dimension::Auto,
            max_width: Dimension::Auto,
            max_height: Dimension::Auto,
            grow: 0.0,
            shrink: 1.0,
            basis: Dimension::Auto,
            position: Position::Relative,
            inset: Edges::all(Dimension::Auto),
            overflow_x: Overflow::Visible,
            overflow_y: Overflow::Visible,
            hidden: false,
            aspect_ratio: None,
            grid: None,
        }
    }
}

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

/// Size hints for boxes whose size does not come from children.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct IntrinsicSize {
    pub min: Option<Size>,
    pub preferred: Option<Size>,
    pub max: Option<Size>,
}
