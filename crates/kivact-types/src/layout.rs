//! Role: Layout configuration, leaf measurement and overlay constraints.
use crate::length::Length;
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

/// Where a surface opens: a [`Placement`] side, with an optional
/// [`OverlayAlign`] and offset. A bare `Placement` converts into one with
/// the overlay's own alignment and distance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfacePlacement {
    /// The side of the anchor the surface asks for.
    pub side: Placement,
    /// Where along that side it sits (`None`: the overlay's own alignment).
    pub align: Option<OverlayAlign>,
    /// The gap from the anchor in logical px (`None`: the overlay's own offset).
    pub offset: Option<f64>,
}

impl From<Placement> for SurfacePlacement {
    fn from(side: Placement) -> Self {
        Self {
            side,
            align: None,
            offset: None,
        }
    }
}

impl Placement {
    /// The side with its alignment along the anchor's edge.
    pub fn align(self, align: OverlayAlign) -> SurfacePlacement {
        SurfacePlacement::from(self).align(align)
    }

    /// The side with its gap from the anchor, in logical px.
    pub fn offset(self, offset: f64) -> SurfacePlacement {
        SurfacePlacement::from(self).offset(offset)
    }
}

impl SurfacePlacement {
    /// Aligns the surface along the anchor's edge.
    pub fn align(self, align: OverlayAlign) -> Self {
        Self {
            align: Some(align),
            ..self
        }
    }

    /// Leaves this gap from the anchor, in logical px.
    pub fn offset(self, offset: f64) -> Self {
        Self {
            offset: Some(offset),
            ..self
        }
    }
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
    pub flow: GridAutoFlow,
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
    Size(Length),
    /// A share of the free space (CSS `fr`; as a minimum, `auto`).
    Fr(f64),
    /// CSS `fit-content(limit)` (as a minimum, `auto`).
    FitContent(Length),
}

/// CSS `grid-auto-flow`: where auto-placed items go.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GridAutoFlow {
    /// Filling each row (the default).
    #[default]
    Row,
    /// Filling each column.
    Column,
    /// Filling rows, back-filling earlier holes with later items.
    RowDense,
    /// Filling columns, back-filling holes.
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
    Definite(f64),
    /// As little as the content allows.
    MinContent,
    /// As much as the content wants.
    MaxContent,
}

/// What a leaf is measured under.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeasureRequest {
    /// Width the layout already fixed.
    pub known_width: Option<f64>,
    /// Height the layout already fixed.
    pub known_height: Option<f64>,
    /// Width available.
    pub available_width: AvailableSpace,
    /// Height available.
    pub available_height: AvailableSpace,
}

impl MeasureRequest {
    /// A request for `width` fixed and any height (a host checks whether
    /// new content keeps its size this way, outside a layout pass).
    pub fn at_width(width: f64) -> Self {
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
    FromTop(f64),
    /// This far below the vertical center of the content box (negative:
    /// above it): a line centered in a box taller than itself.
    FromCenter(f64),
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
    pub margin: f64,
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
    pub offset: f64,
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

/// How a node contributes a layout box.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Display {
    #[default]
    Flex,
    Grid,
    Contents,
}

/// Flex or grid layout of a node. Hosts own the layout algorithm.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutProps {
    pub scroll_snap: crate::ScrollSnap,
    pub display: Display,
    /// Axis revealed during entrance.
    pub enter_size: Option<crate::Orientation>,
    /// Axis collapsed during removal.
    pub exit_size: Option<crate::Orientation>,
    /// Animate layout changes through the visual transform.
    pub layout_transition: Option<crate::motion::Transition>,
    /// Timing shared by presence size changes.
    pub size_transition: Option<crate::motion::Transition>,
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
    pub row_gap: Length,
    /// Between columns (CSS `column-gap`).
    pub column_gap: Length,
    /// Space between border and content.
    pub padding: Edges<Length>,
    /// The border's widths in px (the paint's `Border::widths`): like
    /// padding, they take room inside the box (CSS `box-sizing:
    /// border-box`), so a change of width is a layout change and one of
    /// color or style is not.
    pub border: Edges<f64>,
    /// Space outside the border box.
    pub margin: Edges<Length>,
    /// Preferred border-box width.
    pub width: Length,
    /// Preferred border-box height.
    pub height: Length,
    /// Minimum border-box width.
    pub min_width: Length,
    /// Minimum border-box height.
    pub min_height: Length,
    /// Maximum border-box width.
    pub max_width: Length,
    /// Maximum border-box height.
    pub max_height: Length,
    /// Share of positive free space assigned to this flex item.
    pub grow: f64,
    /// Relative shrinking factor when flex space is insufficient.
    pub shrink: f64,
    /// Initial main-axis size of this flex item.
    pub basis: Length,
    /// Positioning mode relative to parent or window.
    pub position: Position,
    /// Offsets from the reference box edges.
    pub inset: Edges<Length>,
    /// Horizontal overflow and scrolling behavior.
    pub overflow_x: Overflow,
    /// Vertical overflow and scrolling behavior.
    pub overflow_y: Overflow,
    /// Whether this box participates in painting and interaction.
    pub visibility: Visibility,
    /// CSS `aspect-ratio` (width / height; `None`: auto).
    pub aspect_ratio: Option<f64>,
    /// Grid container tracks and grid item placement (`None`: neither).
    pub grid: Option<Box<GridProps>>,
}

impl Default for LayoutProps {
    fn default() -> Self {
        LayoutProps {
            enter_size: None,
            exit_size: None,
            size_transition: None,
            scroll_snap: crate::ScrollSnap::None,
            layout_transition: None,
            direction: FlexDirection::Column,
            align_items: None,
            align_self: None,
            justify: None,
            align_content: None,
            wrap: false,
            row_gap: Length::Px(0.0),
            column_gap: Length::Px(0.0),
            padding: Edges::all(Length::Px(0.0)),
            border: Edges::all(0.0),
            margin: Edges::all(Length::Px(0.0)),
            width: Length::Auto,
            height: Length::Auto,
            min_width: Length::Auto,
            min_height: Length::Auto,
            max_width: Length::Auto,
            max_height: Length::Auto,
            grow: 0.0,
            shrink: 1.0,
            basis: Length::Auto,
            position: Position::Relative,
            inset: Edges::all(Length::Auto),
            overflow_x: Overflow::Visible,
            overflow_y: Overflow::Visible,
            visibility: Visibility::Visible,
            aspect_ratio: None,
            display: Display::Flex,
            grid: None,
        }
    }
}

/// Size hints for boxes whose size does not come from children.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct IntrinsicSize {
    pub min: Option<Size>,
    pub preferred: Option<Size>,
    pub max: Option<Size>,
}

/// Whether a layout box is painted and exposed for interaction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Visibility {
    #[default]
    Visible,
    /// Preserve layout while hiding the subtree from paint, accessibility and input.
    Hidden,
}
