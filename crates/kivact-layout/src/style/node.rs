//! Role: Compact layout node styles for taffy computation.
//!
//! The taffy style of one layout node, compact: the box, flex and grid
//! item properties every node has, read by taffy through its style traits
//! (`CoreStyle`, the flexbox traits, `GridItemStyle`), and a grid
//! container's full taffy style in a box (its tracks; most nodes have
//! none). A `taffy::Style` with taffy's grid feature is about 550 bytes
//! per node, this about half.
//!
//! Invariants:
//! - A grid container's box holds the same box properties as the node's
//!   own (taffy reads the container's core style from either), built by
//!   the one conversion (`style`).
//! - The defaults are taffy's (`taffy::Style::DEFAULT`): a row flex box.
//! - `min_size` is the node's own minimum; taffy reads it raised to
//!   `min_width_floor` (an overlay at least as wide as its anchor), in the
//!   grid container's box too.

use taffy::{
    AlignContent, AlignItems, AlignSelf, BoxGenerationMode, CoreStyle, Dimension, Display,
    FlexDirection, FlexWrap, FlexboxContainerStyle, FlexboxItemStyle, GridItemStyle, GridPlacement,
    JustifyContent, LengthPercentage, LengthPercentageAuto, Line, Overflow, Point, Position, Rect,
    Size,
};

use kivact_types::{GridLine, GridPos};

/// A full taffy style (named grid lines are never used): a grid
/// container's (`NodeDisplay::Grid`).
pub(crate) type TStyle = taffy::Style<String>;

/// What a node's box is: none, a flex box, or a grid container with the full
/// taffy style of its tracks (a grid container whose props have none: `None`).
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum NodeDisplay {
    None,
    Flex,
    Grid(Option<Box<TStyle>>),
}

impl NodeDisplay {
    /// The display taffy reads.
    #[inline(always)]
    pub(crate) fn taffy(&self) -> Display {
        match self {
            NodeDisplay::None => Display::None,
            NodeDisplay::Flex => Display::Flex,
            NodeDisplay::Grid(_) => Display::Grid,
        }
    }

    /// The grid container's full style, when this is a grid container with one.
    #[inline(always)]
    pub(crate) fn grid(&self) -> Option<&TStyle> {
        match self {
            NodeDisplay::Grid(Some(grid)) => Some(grid),
            _ => None,
        }
    }

    /// The grid container's full style, when this is a grid container with one.
    #[inline(always)]
    pub(crate) fn grid_mut(&mut self) -> Option<&mut TStyle> {
        match self {
            NodeDisplay::Grid(Some(grid)) => Some(grid),
            _ => None,
        }
    }
}

/// An item's place in both axes of its grid container (most nodes are
/// auto-placed in flex parents and have none).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GridArea {
    pub(crate) column: GridLine,
    pub(crate) row: GridLine,
}

/// The taffy style of one layout node (see the module docs).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NodeStyle {
    pub(crate) display: NodeDisplay,
    pub(crate) position: Position,
    pub(crate) overflow: Point<Overflow>,
    pub(crate) inset: Rect<LengthPercentageAuto>,
    pub(crate) size: Size<Dimension>,
    pub(crate) min_size: Size<LengthPercentageAuto>,
    /// A width taffy's minimum width does not go below (0: none); set by
    /// the layout pass, never by the props.
    pub(crate) min_width_floor: f32,
    pub(crate) max_size: Size<LengthPercentageAuto>,
    // xtask-allow(state-fields): option: the authored property; absent means the taffy default
    pub(crate) aspect_ratio: Option<f32>,
    pub(crate) margin: Rect<LengthPercentageAuto>,
    pub(crate) padding: Rect<LengthPercentage>,
    pub(crate) border: Rect<LengthPercentage>,
    pub(crate) gap: Size<LengthPercentage>,
    pub(crate) flex_direction: FlexDirection,
    pub(crate) flex_wrap: FlexWrap,
    // xtask-allow(state-fields): option: the authored property; absent means the taffy default
    pub(crate) align_items: Option<AlignItems>,
    // xtask-allow(state-fields): option: the authored property; absent means the taffy default
    pub(crate) align_self: Option<AlignSelf>,
    // xtask-allow(state-fields): option: the authored property; absent means the taffy default
    pub(crate) align_content: Option<AlignContent>,
    // xtask-allow(state-fields): option: the authored property; absent means the taffy default
    pub(crate) justify_content: Option<JustifyContent>,
    pub(crate) flex_basis: Dimension,
    pub(crate) flex_grow: f32,
    pub(crate) flex_shrink: f32,
    /// Its place as a grid item (absent: auto-placed).
    pub(crate) grid_area: Option<Box<GridArea>>,
    // xtask-allow(state-fields): option: the authored property; absent means the taffy default
    pub(crate) justify_self: Option<AlignSelf>,
}

const _: () = assert!(
    std::mem::size_of::<NodeStyle>() <= 264,
    "NodeStyle grew: box the new rare field"
);

impl Default for NodeStyle {
    fn default() -> Self {
        let d = TStyle::DEFAULT;
        NodeStyle {
            display: match d.display {
                Display::None => NodeDisplay::None,
                Display::Grid => NodeDisplay::Grid(None),
                _ => NodeDisplay::Flex,
            },
            position: d.position,
            overflow: d.overflow,
            inset: d.inset,
            size: d.size,
            min_size: d.min_size,
            min_width_floor: 0.0,
            max_size: d.max_size,
            aspect_ratio: d.aspect_ratio,
            margin: d.margin,
            padding: d.padding,
            border: d.border,
            gap: d.gap,
            flex_direction: d.flex_direction,
            flex_wrap: d.flex_wrap,
            align_items: d.align_items,
            align_self: d.align_self,
            align_content: d.align_content,
            justify_content: d.justify_content,
            flex_basis: d.flex_basis,
            flex_grow: d.flex_grow,
            flex_shrink: d.flex_shrink,
            grid_area: None,
            justify_self: d.justify_self,
        }
    }
}

impl NodeStyle {
    /// The minimum size taffy reads: the own one, its width raised to the
    /// floor (a length below it, `auto` or a percentage becomes the floor).
    fn floored_min_size(&self) -> Size<LengthPercentageAuto> {
        let floor = self.min_width_floor;
        let own = self.min_size.width.into_raw();
        let width = if floor <= 0.0 {
            self.min_size.width
        } else if own.tag() == taffy::CompactLength::LENGTH_TAG {
            LengthPercentageAuto::length(own.value().max(floor))
        } else {
            LengthPercentageAuto::length(floor)
        };
        Size {
            width,
            height: self.min_size.height,
        }
    }

    /// Raise the minimum width to at least `floor` (0: the own minimum).
    /// Returns whether it changed, so the caller drops the node's cache.
    pub(crate) fn set_min_width_floor(&mut self, floor: f32) -> bool {
        if self.min_width_floor == floor {
            return false;
        }
        self.min_width_floor = floor;
        let min_size = self.floored_min_size();
        if let Some(grid) = self.display.grid_mut() {
            grid.min_size = min_size;
        }
        true
    }

    /// Replace the box limits in both forms of the style for an overlay pass.
    pub(crate) fn set_size_bounds(
        &mut self,
        min: Size<LengthPercentageAuto>,
        max: Size<LengthPercentageAuto>,
    ) {
        self.min_size = min;
        self.max_size = max;
        let min = self.floored_min_size();
        if let Some(grid) = self.display.grid_mut() {
            grid.min_size = min;
            grid.max_size = max;
        }
    }
}

/// An item's place along one axis as taffy reads it.
fn line(l: GridLine) -> Line<GridPlacement<String>> {
    let pos = |p: GridPos| match p {
        GridPos::Auto | GridPos::Line(0) => GridPlacement::Auto,
        GridPos::Line(n) => taffy::style_helpers::line(n),
        GridPos::Span(n) => GridPlacement::Span(n.max(1)),
    };
    Line {
        start: pos(l.start),
        end: pos(l.end),
    }
}

impl CoreStyle for NodeStyle {
    type CustomIdent = String;

    #[inline(always)]
    fn box_generation_mode(&self) -> BoxGenerationMode {
        match self.display {
            NodeDisplay::None => BoxGenerationMode::None,
            NodeDisplay::Flex | NodeDisplay::Grid(_) => BoxGenerationMode::Normal,
        }
    }

    #[inline(always)]
    fn overflow(&self) -> Point<Overflow> {
        self.overflow
    }

    #[inline(always)]
    fn scrollbar_width(&self) -> f32 {
        0.0
    }

    #[inline(always)]
    fn position(&self) -> Position {
        self.position
    }

    #[inline(always)]
    fn inset(&self) -> Rect<LengthPercentageAuto> {
        self.inset
    }

    #[inline(always)]
    fn size(&self) -> Size<Dimension> {
        self.size
    }

    #[inline(always)]
    fn min_size(&self) -> Size<LengthPercentageAuto> {
        self.floored_min_size()
    }

    #[inline(always)]
    fn max_size(&self) -> Size<LengthPercentageAuto> {
        self.max_size
    }

    #[inline(always)]
    fn aspect_ratio(&self) -> Option<f32> {
        self.aspect_ratio
    }

    #[inline(always)]
    fn margin(&self) -> Rect<LengthPercentageAuto> {
        self.margin
    }

    #[inline(always)]
    fn padding(&self) -> Rect<LengthPercentage> {
        self.padding
    }

    #[inline(always)]
    fn border(&self) -> Rect<LengthPercentage> {
        self.border
    }
}

impl FlexboxContainerStyle for NodeStyle {
    #[inline(always)]
    fn flex_direction(&self) -> FlexDirection {
        self.flex_direction
    }

    #[inline(always)]
    fn flex_wrap(&self) -> FlexWrap {
        self.flex_wrap
    }

    #[inline(always)]
    fn gap(&self) -> Size<LengthPercentage> {
        self.gap
    }

    #[inline(always)]
    fn align_content(&self) -> Option<AlignContent> {
        self.align_content
    }

    #[inline(always)]
    fn align_items(&self) -> Option<AlignItems> {
        self.align_items
    }

    #[inline(always)]
    fn justify_content(&self) -> Option<JustifyContent> {
        self.justify_content
    }
}

impl FlexboxItemStyle for NodeStyle {
    #[inline(always)]
    fn flex_basis(&self) -> Dimension {
        self.flex_basis
    }

    #[inline(always)]
    fn flex_grow(&self) -> f32 {
        self.flex_grow
    }

    #[inline(always)]
    fn flex_shrink(&self) -> f32 {
        self.flex_shrink
    }

    #[inline(always)]
    fn align_self(&self) -> Option<AlignSelf> {
        self.align_self
    }
}

impl GridItemStyle for NodeStyle {
    #[inline(always)]
    fn grid_row(&self) -> Line<GridPlacement<String>> {
        line(
            self.grid_area
                .as_deref()
                .map_or(GridLine::default(), |a| a.row),
        )
    }

    #[inline(always)]
    fn grid_column(&self) -> Line<GridPlacement<String>> {
        line(
            self.grid_area
                .as_deref()
                .map_or(GridLine::default(), |a| a.column),
        )
    }

    #[inline(always)]
    fn align_self(&self) -> Option<AlignSelf> {
        self.align_self
    }

    #[inline(always)]
    fn justify_self(&self) -> Option<AlignSelf> {
        self.justify_self
    }
}
