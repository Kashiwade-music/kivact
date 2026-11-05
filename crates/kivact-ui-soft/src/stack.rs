//! Role: Provide Soft theme spacing and flex layout primitives.
use kivact::state::IntoValue;
use kivact::state::Value;

use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::ScrollAxes;
use kivact::view::node;
use kivact::view::scroll;

use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Length;
use kivact::style::Style;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::ReadingDirection;
use kivact::view::View;
use kivact::view::use_direction;

/// Theme spacing steps, resolved through the current Soft tokens.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Space {
    #[default]
    /// Resolve the theme’s `s0` spacing token.
    S0,
    /// Resolve the theme’s `s0_5` spacing token.
    S0_5,
    /// Resolve the theme’s `s1` spacing token.
    S1,
    /// Resolve the theme’s `s1_5` spacing token.
    S1_5,
    /// Resolve the theme’s `s2` spacing token.
    S2,
    /// Resolve the theme’s `s3` spacing token.
    S3,
    /// Resolve the theme’s `s4` spacing token.
    S4,
    /// Resolve the theme’s `s5` spacing token.
    S5,
    /// Resolve the theme’s `s6` spacing token.
    S6,
    /// Resolve the theme’s `s8` spacing token.
    S8,
    /// Resolve the theme’s `s10` spacing token.
    S10,
}

impl Space {
    /// Resolve this step in logical pixels from the current reactive theme scope.
    pub fn px(self) -> f64 {
        let s = kivact::style::use_tokens::<crate::Tokens>().spacing;
        match self {
            Self::S0 => s.s0,
            Self::S0_5 => s.s0_5,
            Self::S1 => s.s1,
            Self::S1_5 => s.s1_5,
            Self::S2 => s.s2,
            Self::S3 => s.s3,
            Self::S4 => s.s4,
            Self::S5 => s.s5,
            Self::S6 => s.s6,
            Self::S8 => s.s8,
            Self::S10 => s.s10,
        }
    }
}

pub(crate) fn rtl() -> bool {
    use_direction().get() == ReadingDirection::Rtl
}

#[derive(Clone, Copy, Debug, Default)]
// xtask-allow(state-fields): Passive optional logical padding values resolved by precedence; these numerical style fields are independent configuration, not runtime flags.
pub(crate) struct Padding {
    pub all: Option<Space>,
    pub x: Option<Space>,
    pub y: Option<Space>,
    pub start: Option<Space>,
    pub end: Option<Space>,
    pub top: Option<Space>,
    pub bottom: Option<Space>,
}

/// Logical insets announced by containers to their descendants.
#[derive(Clone, Copy, Debug, Default)]
pub struct ContainerInset {
    /// Leading inset in logical pixels, following reading direction.
    pub start: f64,
    /// Trailing inset in logical pixels, following reading direction.
    pub end: f64,
    /// Top inset in logical pixels.
    pub top: f64,
    /// Bottom inset in logical pixels.
    pub bottom: f64,
}

impl ContainerInset {
    pub(crate) fn uniform(px: f64) -> Self {
        Self {
            start: px,
            end: px,
            top: px,
            bottom: px,
        }
    }

    pub(crate) fn sides(self) -> (f64, f64) {
        if rtl() {
            (self.end, self.start)
        } else {
            (self.start, self.end)
        }
    }

    pub(crate) fn apply(self, style: Style) -> Style {
        let (left, right) = self.sides();
        style
            .padding_left(left)
            .padding_right(right)
            .padding_top(self.top)
            .padding_bottom(self.bottom)
    }
}

impl Padding {
    pub(crate) fn resolve(self, default: f64) -> ContainerInset {
        let side =
            |edge: Option<Space>, axis| edge.or(axis).or(self.all).map_or(default, Space::px);
        ContainerInset {
            start: side(self.start, self.x),
            end: side(self.end, self.x),
            top: side(self.top, self.y),
            bottom: side(self.bottom, self.y),
        }
    }
}

macro_rules! padding_options {
    () => {
        /// Set all padding edges to a theme spacing step; more specific edges take precedence.
        pub fn padding(mut self, value: Space) -> Self {
            self.padding.all = Some(value);
            self
        }
        /// Set both inline padding edges unless individually overridden.
        pub fn padding_x(mut self, value: Space) -> Self {
            self.padding.x = Some(value);
            self
        }
        /// Set top and bottom padding unless individually overridden.
        pub fn padding_y(mut self, value: Space) -> Self {
            self.padding.y = Some(value);
            self
        }
        /// Set logical leading padding, following reading direction.
        pub fn padding_start(mut self, value: Space) -> Self {
            self.padding.start = Some(value);
            self
        }
        /// Set logical trailing padding, following reading direction.
        pub fn padding_end(mut self, value: Space) -> Self {
            self.padding.end = Some(value);
            self
        }
        /// Set top padding to a theme spacing step.
        pub fn padding_top(mut self, value: Space) -> Self {
            self.padding.top = Some(value);
            self
        }
        /// Set bottom padding to a theme spacing step.
        pub fn padding_bottom(mut self, value: Space) -> Self {
            self.padding.bottom = Some(value);
            self
        }
        /// Set both logical inline padding edges.
        pub fn padding_inline(self, value: Space) -> Self {
            self.padding_x(value)
        }
        /// Set both block padding edges: top and bottom.
        pub fn padding_block(self, value: Space) -> Self {
            self.padding_y(value)
        }
        /// Set the logical leading padding edge.
        pub fn padding_inline_start(self, value: Space) -> Self {
            self.padding_start(value)
        }
        /// Set the logical trailing padding edge.
        pub fn padding_inline_end(self, value: Space) -> Self {
            self.padding_end(value)
        }
        /// Set the top padding edge.
        pub fn padding_block_start(self, value: Space) -> Self {
            self.padding_top(value)
        }
        /// Set the bottom padding edge.
        pub fn padding_block_end(self, value: Space) -> Self {
            self.padding_bottom(value)
        }
    };
}
pub(crate) use padding_options;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Alignment on a named stack axis; distribution modes stretch on the cross axis.
pub enum StackAlign {
    /// Pack children toward the axis start.
    Start,
    /// Center children on the axis.
    Center,
    /// Pack children toward the axis end.
    End,
    /// Distribute free space between children on the main axis.
    Between,
    /// Distribute main-axis space around children, with half-size end spaces.
    Around,
    /// Distribute equal main-axis space between children and at both ends.
    Evenly,
    /// Stretch alignment across the available axis.
    Stretch,
}

impl StackAlign {
    fn main(self) -> Justify {
        match self {
            Self::Start => Justify::Start,
            Self::Center => Justify::Center,
            Self::End => Justify::End,
            Self::Between => Justify::SpaceBetween,
            Self::Around => Justify::SpaceAround,
            Self::Evenly => Justify::SpaceEvenly,
            Self::Stretch => Justify::Stretch,
        }
    }

    fn cross(self) -> Align {
        match self {
            Self::Start => Align::Start,
            Self::Center => Align::Center,
            Self::End => Align::End,
            _ => Align::Stretch,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Whether children remain on one flex line or can wrap.
pub enum StackWrap {
    #[default]
    /// Keep children on one flex line; the default.
    NoWrap,
    /// Allow children to flow onto additional flex lines.
    Wrap,
}

impl From<bool> for StackWrap {
    fn from(wrap: bool) -> Self {
        if wrap { Self::Wrap } else { Self::NoWrap }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Axes centered by `FlexStack::axis`; interpreted against the stack’s main/cross axes.
pub enum CenterAxis {
    #[default]
    /// Center both main and cross axes; the default.
    Both,
    /// Center the main axis and stretch the cross axis.
    Horizontal,
    /// Center the cross axis and start-align the main axis.
    Vertical,
}

/// A direction-aware flex container.
pub struct FlexStack {
    root: Node,
    direction: Value<Direction>,
    gap: Value<Space>,
    padding: Padding,
    align: Value<Align>,
    justify: Value<Justify>,
    wrap: StackWrap,
    scrollable: Value<bool>,
    horizontal: Option<StackAlign>,
    vertical: Option<StackAlign>,
}

/// Create a vertical flex stack with zero gap and padding.
pub fn flex_stack(children: impl IntoView) -> FlexStack {
    vstack(children)
}

/// Create a reading-direction-aware horizontal stack with zero gap and padding.
pub fn hstack(children: impl IntoView) -> FlexStack {
    vstack(children).direction(Direction::Row)
}

/// Create a vertical stack with stretched children and leading main-axis alignment.
pub fn vstack(children: impl IntoView) -> FlexStack {
    FlexStack {
        root: node().child(children),
        direction: (Direction::Column).into_value(),
        gap: (Space::S0).into_value(),
        padding: Padding::default(),
        align: (Align::Stretch).into_value(),
        justify: (Justify::Start).into_value(),
        wrap: StackWrap::NoWrap,
        scrollable: (false).into_value(),
        horizontal: None,
        vertical: None,
    }
}

/// A horizontal stack centered on both axes.
pub fn center(children: impl IntoView) -> FlexStack {
    hstack(children)
        .align(Align::Center)
        .justify(Justify::Center)
}

impl FlexStack {
    padding_options!();
    /// Set constant or reactive flex direction; horizontal directions follow reading direction.
    pub fn direction(mut self, direction: impl IntoValue<Direction>) -> Self {
        self.direction = direction.into_value();
        self
    }
    /// Set constant or reactive theme spacing between children; defaults to zero.
    pub fn gap(mut self, gap: impl IntoValue<Space>) -> Self {
        self.gap = gap.into_value();
        self
    }
    /// Set constant or reactive cross-axis alignment; defaults to stretch.
    pub fn align(mut self, align: impl IntoValue<Align>) -> Self {
        self.align = align.into_value();
        self
    }
    /// Set constant or reactive main-axis distribution; defaults to start.
    pub fn justify(mut self, justify: impl IntoValue<Justify>) -> Self {
        self.justify = justify.into_value();
        self
    }
    /// Enable flex wrapping with a wrap mode or bool; disabled by default.
    pub fn wrap(mut self, wrap: impl Into<StackWrap>) -> Self {
        self.wrap = wrap.into();
        self
    }
    /// Choose at view construction whether to scroll along the initial main axis.
    pub fn scrollable(mut self, scrollable: impl IntoValue<bool>) -> Self {
        self.scrollable = scrollable.into_value();
        self
    }
    /// Override horizontal alignment, mapped to the initial main or cross axis.
    pub fn h_align(mut self, align: StackAlign) -> Self {
        self.horizontal = Some(align);
        self
    }
    /// Override vertical alignment, mapped to the initial main or cross axis.
    pub fn v_align(mut self, align: StackAlign) -> Self {
        self.vertical = Some(align);
        self
    }
    /// Center main, cross, or both axes; horizontal selects main and vertical selects cross.
    pub fn axis(mut self, axis: CenterAxis) -> Self {
        self.justify = (if axis == CenterAxis::Vertical {
            Justify::Start
        } else {
            Justify::Center
        })
        .into_value();
        self.align = (if axis == CenterAxis::Horizontal {
            Align::Stretch
        } else {
            Align::Center
        })
        .into_value();
        self
    }
    /// When true, disable growth and align this stack to the parent’s cross-axis start.
    pub fn inline(mut self, inline: bool) -> Self {
        if inline {
            self.root = self.root.style(style().grow(0).align_self(Align::Start));
        }
        self
    }
    /// Set a constant or reactive width using a logical length.
    pub fn width(mut self, width: impl IntoValue<Length>) -> Self {
        let width = width.into_value();
        self.root = self.root.style(move || style().width(width.get()));
        self
    }
    /// Set a constant or reactive height using a logical length.
    pub fn height(mut self, height: impl IntoValue<Length>) -> Self {
        let height = height.into_value();
        self.root = self.root.style(move || style().height(height.get()));
        self
    }
}

impl NodeBuilder for FlexStack {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for FlexStack {
    fn into_view(self) -> View {
        let Self {
            root,
            direction,
            gap,
            padding,
            align,
            justify,
            wrap,
            scrollable,
            horizontal,
            vertical,
        } = self;
        let is_row = matches!(direction.get(), Direction::Row | Direction::RowReverse);
        let (main, cross) = if is_row {
            (horizontal, vertical)
        } else {
            (vertical, horizontal)
        };
        let root = root.with_node(move |node| {
            node.base_style(move || {
                let direction = match (direction.get(), rtl()) {
                    (Direction::Row, true) => Direction::RowReverse,
                    (Direction::RowReverse, true) => Direction::Row,
                    _ => direction.get(),
                };
                padding.resolve(0.0).apply(
                    style()
                        .direction(direction)
                        .gap(gap.get().px())
                        .align_items(cross.map_or(align.get(), StackAlign::cross))
                        .justify_content(main.map_or(justify.get(), StackAlign::main))
                        .wrap(wrap == StackWrap::Wrap),
                )
            })
        });
        if scrollable.get() {
            stack_scroll(
                root,
                if is_row {
                    ScrollAxes::Horizontal
                } else {
                    ScrollAxes::Vertical
                },
            )
        } else {
            root.into_view()
        }
    }
}

fn stack_scroll(root: Node, axes: ScrollAxes) -> View {
    let mut viewport = scroll(()).axes(axes);
    *viewport.node_mut() = root;
    viewport.into_view()
}

/// Flex sizing for a stack item.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StackItemSize {
    /// Allow shrinking without growing.
    Auto,
    /// Grow into free space and allow shrinking.
    Fill,
    #[default]
    /// Neither grow nor shrink; the default.
    Static,
}

/// Flex child wrapper with explicit sizing and optional vertical scrolling.
pub struct StackItem {
    root: Node,
    size: Value<StackItemSize>,
    scrollable: Value<bool>,
}

/// Create a non-growing, non-shrinking stack child without scrolling.
pub fn stack_item(children: impl IntoView) -> StackItem {
    StackItem {
        root: node().child(children),
        size: (StackItemSize::Static).into_value(),
        scrollable: (false).into_value(),
    }
}

impl StackItem {
    /// Choose flex sizing at view construction; defaults to static.
    pub fn size(mut self, size: impl IntoValue<StackItemSize>) -> Self {
        self.size = size.into_value();
        self
    }
    /// Choose at view construction whether this child has a vertical scroll viewport.
    pub fn scrollable(mut self, scrollable: impl IntoValue<bool>) -> Self {
        self.scrollable = scrollable.into_value();
        self
    }
    /// Override this child’s alignment on its parent’s cross axis.
    pub fn cross_align_self(self, align: Align) -> Self {
        self.align_self(align)
    }
    /// Override this child’s alignment on its parent’s cross axis.
    pub fn align_self(mut self, align: Align) -> Self {
        self.root = self.root.style(style().align_self(align));
        self
    }
}

impl NodeBuilder for StackItem {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for StackItem {
    fn into_view(self) -> View {
        let sizing = match self.size.get() {
            StackItemSize::Auto => style().grow(0).shrink(1),
            StackItemSize::Fill => style().grow(1).shrink(1),
            StackItemSize::Static => style().grow(0).shrink(0),
        };
        let root = self
            .root
            .with_node(move |node| node.base_style(sizing.min_width(0).min_height(0)));
        if self.scrollable.get() {
            stack_scroll(root, ScrollAxes::Vertical)
        } else {
            root.into_view()
        }
    }
}

kivact::state::constant_value!(Space, StackItemSize);
