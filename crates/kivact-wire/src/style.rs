//! Role: Resolved (theme-free) style props.
//! Clients resolve styles and theme tokens into these values; hosts apply
//! them to their layout and paint implementations.

use super::Point;
use super::grid::GridProps;
use super::paint::{Border, Corners, Effects, Fill, Len, Ring};
use super::values::{Dimension, Edges, Rgba};
use super::{Align, FlexDirection, Justify, Overflow, Position};

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

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Shadow {
    pub offset: Point,
    pub blur: f32,
    pub spread: f32,
    pub color: Rgba,
    /// Inside the box (CSS `inset`).
    pub inset: bool,
}

/// Element transform: CSS `translate`, `rotate`, `scale` and `transform`
/// with `transform-origin`, in one canonical form. About `origin`, the
/// box is scaled, skewed along x, rotated and moved (CSS
/// `translate(..) rotate(..) skewX(..) scale(..)`); it maps the node's box
/// and its whole subtree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElementTransform {
    /// The move in px ...
    pub translate: Point,
    /// ... plus fractions of the box's own size: x moves by
    /// `per_size[0] × width + per_size[1] × height`, y by
    /// `per_size[2] × width + per_size[3] × height` (a percentage
    /// translation, mapped by what precedes it in the CSS list).
    pub per_size: [f32; 4],
    /// Clockwise, in degrees.
    pub rotate: f32,
    /// Along x, in degrees.
    pub skew_x: f32,
    pub scale: Point,
    /// Against the box's width and height (CSS `transform-origin`,
    /// default `50% 50%`).
    pub origin: [Len; 2],
}

impl ElementTransform {
    /// No transform (origin at the centre).
    pub const NONE: ElementTransform = ElementTransform {
        translate: Point { x: 0.0, y: 0.0 },
        per_size: [0.0; 4],
        rotate: 0.0,
        skew_x: 0.0,
        scale: Point { x: 1.0, y: 1.0 },
        origin: [Len::pct(50.0), Len::pct(50.0)],
    };

    /// Whether it moves nothing (whatever its origin).
    pub fn is_none(&self) -> bool {
        self.translate == Point::default()
            && self.per_size == [0.0; 4]
            && self.rotate == 0.0
            && self.skew_x == 0.0
            && self.scale == Point { x: 1.0, y: 1.0 }
    }
}

impl Default for ElementTransform {
    fn default() -> Self {
        ElementTransform::NONE
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaintProps {
    /// Optional host-driven paint transition, absent on ordinary nodes.
    pub transition: Option<std::sync::Arc<super::motion::Transition>>,
    pub background: Option<Fill>,
    pub border: Option<Border>,
    /// Focus ring drawn outside the border box.
    pub ring: Option<Ring>,
    pub radius: Corners,
    /// CSS order: the first is on top.
    pub shadows: Vec<Shadow>,
    /// Group opacity of the node and its subtree.
    pub opacity: f32,
    pub transform: ElementTransform,
    /// The paint and hit order among its siblings (CSS `z-index`; ties
    /// keep the tree order).
    pub z_index: i32,
    pub cursor: Cursor,
    pub scrollbar: Option<Box<ScrollbarProps>>,
    /// Clip children to the padding box (rounded by the inner radii).
    pub clip: bool,
    /// Filters, backdrop, blending, mask and clip path (`None`: none).
    pub effects: Option<Box<Effects>>,
}

impl Default for PaintProps {
    fn default() -> Self {
        PaintProps {
            transition: None,
            background: None,
            border: None,
            ring: None,
            radius: Corners::default(),
            shadows: Vec::new(),
            opacity: 1.0,
            transform: ElementTransform::NONE,
            z_index: 0,
            cursor: Cursor::Auto,
            scrollbar: None,
            clip: false,
            effects: None,
        }
    }
}

/// A partial paint override applied by the host while a pointer/focus state
/// is active. Layout never changes with these states.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct PaintDelta {
    pub background: Option<Fill>,
    pub border: Option<Border>,
    pub ring: Option<Ring>,
    pub radius: Option<Corners>,
    pub opacity: Option<f32>,
    pub shadows: Option<Vec<Shadow>>,
    pub transform: Option<ElementTransform>,
    pub z_index: Option<i32>,
    pub cursor: Option<Cursor>,
    pub scrollbar: Option<Box<ScrollbarDelta>>,
    /// Replaces the node's effects.
    pub effects: Option<Option<Box<Effects>>>,
    pub text_color: Option<Rgba>,
}

/// Host-evaluated state styles. `disabled` and `selected` are resolved by the
/// core because the core owns those states.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct StateStyles {
    pub hover: Option<PaintDelta>,
    pub pressed: Option<PaintDelta>,
    pub focus: Option<PaintDelta>,
    /// Applied over `focus` while the focus is visible (the host asks for a
    /// focus indication: keyboard focus, not a pointer press).
    pub focus_visible: Option<PaintDelta>,
    /// Applied while the nearest ancestor that is a `group` is hovered (in
    /// the hover chain: the pointer is over it or a node below it), under
    /// the node's own `hover`.
    pub group_hover: Option<PaintDelta>,
    /// Applied while the focus is on the nearest ancestor that is a `group`
    /// or on a node below it, under the node's own `focus`.
    pub group_focus: Option<PaintDelta>,
    /// The node is a group for the `group_hover` and `group_focus` of the
    /// nodes below it.
    pub group: bool,
    /// The paint the node starts from when it is first attached (CSS
    /// `@starting-style`): its paint transition moves it to its paint. A
    /// node without a paint transition, or under reduced motion with a
    /// decorative one, shows its paint at once.
    pub enter: Option<PaintDelta>,
    /// The paint a removed node moves to by its paint transition before it
    /// goes: until every exit in the removed subtree ends, the subtree is
    /// drawn where it was but takes no hit, focus or accessibility node.
    /// Without a transition (or under reduced motion with a decorative
    /// one) the node goes at once.
    pub exit: Option<PaintDelta>,
}

impl StateStyles {
    /// Every delta it sets.
    pub fn deltas(&self) -> impl Iterator<Item = &PaintDelta> {
        [
            &self.hover,
            &self.pressed,
            &self.focus,
            &self.focus_visible,
            &self.group_hover,
            &self.group_focus,
            &self.enter,
            &self.exit,
        ]
        .into_iter()
        .flatten()
    }
}

pub use kivact_types::Cursor;

/// Overlay scrollbar appearance; layout space is never reserved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarProps {
    /// Preferred border-box width.
    pub width: f32,
    pub thumb: Rgba,
    pub track: Rgba,
}
/// Sparse scrollbar state style; unset fields preserve the base style.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollbarDelta {
    /// Preferred border-box width.
    pub width: Option<f32>,
    pub colors: Option<(Rgba, Rgba)>,
}
impl Default for ScrollbarProps {
    fn default() -> Self {
        Self {
            width: 6.0,
            thumb: Rgba {
                r: 0.45,
                g: 0.48,
                b: 0.52,
                a: 0.65,
            },
            track: Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
        }
    }
}
