//! Role: Provide Soft theme spacing and flex layout primitives.
use kivact::component;
use kivact::state::IntoValue;
use kivact::state::Value;

use kivact::view::Element;
use kivact::view::Node;
use kivact::view::ScrollAxes;
use kivact::view::View;
use kivact::view::node;
use kivact::view::scroll;

use crate::space::Space;
use kivact::Prop;
use kivact::style::Align;
use kivact::style::FlexDirection;
use kivact::style::Justify;
use kivact::style::Length;
use kivact::style::Style;
use kivact::style::style;
use kivact::view::ReadingDirection;
use kivact::view::use_direction;

pub(crate) fn rtl() -> bool {
    use_direction() == ReadingDirection::Rtl
}

#[derive(Clone, Copy, Debug, Default)]
// Passive optional logical padding values resolved by precedence; these numerical style fields are independent configuration, not runtime flags.
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
/// Axes centered by a stack's `axis` prop; interpreted against the stack's main/cross axes.
pub enum CenterAxis {
    #[default]
    /// Center both main and cross axes; the default.
    Both,
    /// Center the main axis and stretch the cross axis.
    Horizontal,
    /// Center the cross axis and start-align the main axis.
    Vertical,
}

/// The settings that the two stack components share.
struct FlexSpec {
    direction: FlexDirection,
    gap: Value<Space>,
    padding: Padding,
    align: Value<Align>,
    justify: Value<Justify>,
    wrap: StackWrap,
    scrollable: Value<bool>,
    horizontal: Option<StackAlign>,
    vertical: Option<StackAlign>,
    inline: bool,
    width: Option<Value<Length>>,
    height: Option<Value<Length>>,
}

/// Center the main, cross, or both axes; `None` keeps the given alignment.
fn apply_axis(
    axis: Option<CenterAxis>,
    justify: Value<Justify>,
    align: Value<Align>,
) -> (Value<Justify>, Value<Align>) {
    let Some(axis) = axis else {
        return (justify, align);
    };
    let justify = if axis == CenterAxis::Vertical {
        Justify::Start
    } else {
        Justify::Center
    };
    let align = if axis == CenterAxis::Horizontal {
        Align::Stretch
    } else {
        Align::Center
    };
    (justify.into_value(), align.into_value())
}

/// Build the root node of a flex stack around its children.
fn flex_root(children: View, spec: FlexSpec) -> Node {
    let FlexSpec {
        direction,
        gap,
        padding,
        align,
        justify,
        wrap,
        scrollable,
        horizontal,
        vertical,
        inline,
        width,
        height,
    } = spec;
    let is_row = matches!(direction, FlexDirection::Row | FlexDirection::RowReverse);
    let (main, cross) = if is_row {
        (horizontal, vertical)
    } else {
        (vertical, horizontal)
    };
    let root = node().child(children).with_node(move |node| {
        node.base_style(move || {
            let direction = match (direction, rtl()) {
                (FlexDirection::Row, true) => FlexDirection::RowReverse,
                (FlexDirection::RowReverse, true) => FlexDirection::Row,
                (direction, _) => direction,
            };
            padding.resolve(0.0).apply(
                style()
                    .flex_direction(direction)
                    .gap(gap.get().px())
                    .align_items(cross.map_or(align.get(), StackAlign::cross))
                    .justify_content(main.map_or(justify.get(), StackAlign::main))
                    .flex_wrap(wrap == StackWrap::Wrap),
            )
        })
    });
    let root = if scrollable.get() {
        let axes = if is_row {
            ScrollAxes::Horizontal
        } else {
            ScrollAxes::Vertical
        };
        node().child(scroll(root).axes(axes))
    } else {
        root
    };
    let root = if inline {
        root.style(style().flex_grow(0).align_self(Align::Start))
    } else {
        root
    };
    let root = match width {
        Some(width) => root.style(move || style().width(width.get())),
        None => root,
    };
    match height {
        Some(height) => root.style(move || style().height(height.get())),
        None => root,
    }
}

/// A horizontal stack: children laid out in a row that follows reading direction.
#[component]
pub fn hstack(
    children: View,
    #[prop(default = Space::S0)] gap: Value<Space>,
    padding: Option<Space>,
    padding_x: Option<Space>,
    padding_y: Option<Space>,
    padding_start: Option<Space>,
    padding_end: Option<Space>,
    padding_top: Option<Space>,
    padding_bottom: Option<Space>,
    #[prop(default = Align::Stretch)] align: Value<Align>,
    #[prop(default = Justify::Start)] justify: Value<Justify>,
    #[prop(default = StackWrap::NoWrap)] wrap: StackWrap,
    #[prop(default = false)] scrollable: Value<bool>,
    h_align: Option<StackAlign>,
    v_align: Option<StackAlign>,
    axis: Option<CenterAxis>,
    #[prop(default = false)] inline: bool,
    width: Option<Value<Length>>,
    height: Option<Value<Length>>,
) -> impl Element {
    let (justify, align) = apply_axis(axis, justify, align);
    flex_root(
        children,
        FlexSpec {
            direction: FlexDirection::Row,
            gap,
            padding: Padding {
                all: padding,
                x: padding_x,
                y: padding_y,
                start: padding_start,
                end: padding_end,
                top: padding_top,
                bottom: padding_bottom,
            },
            align,
            justify,
            wrap,
            scrollable,
            horizontal: h_align,
            vertical: v_align,
            inline,
            width,
            height,
        },
    )
}

/// A vertical stack with stretched children and leading main-axis alignment.
#[component]
pub fn vstack(
    children: View,
    #[prop(default = Space::S0)] gap: Value<Space>,
    padding: Option<Space>,
    padding_x: Option<Space>,
    padding_y: Option<Space>,
    padding_start: Option<Space>,
    padding_end: Option<Space>,
    padding_top: Option<Space>,
    padding_bottom: Option<Space>,
    #[prop(default = Align::Stretch)] align: Value<Align>,
    #[prop(default = Justify::Start)] justify: Value<Justify>,
    #[prop(default = StackWrap::NoWrap)] wrap: StackWrap,
    #[prop(default = false)] scrollable: Value<bool>,
    h_align: Option<StackAlign>,
    v_align: Option<StackAlign>,
    axis: Option<CenterAxis>,
    #[prop(default = false)] inline: bool,
    width: Option<Value<Length>>,
    height: Option<Value<Length>>,
) -> impl Element {
    let (justify, align) = apply_axis(axis, justify, align);
    flex_root(
        children,
        FlexSpec {
            direction: FlexDirection::Column,
            gap,
            padding: Padding {
                all: padding,
                x: padding_x,
                y: padding_y,
                start: padding_start,
                end: padding_end,
                top: padding_top,
                bottom: padding_bottom,
            },
            align,
            justify,
            wrap,
            scrollable,
            horizontal: h_align,
            vertical: v_align,
            inline,
            width,
            height,
        },
    )
}

/// A horizontal stack centered on both axes.
#[component]
pub fn center(children: View) -> impl Element {
    hstack(children)
        .align(Align::Center)
        .justify(Justify::Center)
}

/// Flex sizing for a stack item.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
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
#[component]
pub fn stack_item(
    children: View,
    #[prop(default = StackItemSize::Static)] size: Value<StackItemSize>,
    #[prop(default = false)] scrollable: Value<bool>,
    align_self: Option<Align>,
) -> impl Element {
    let sizing = match size.get() {
        StackItemSize::Auto => style().flex_grow(0).flex_shrink(1),
        StackItemSize::Fill => style().flex_grow(1).flex_shrink(1),
        StackItemSize::Static => style().flex_grow(0).flex_shrink(0),
    };
    let root = node()
        .child(children)
        .with_node(move |node| node.base_style(sizing.min_width(0).min_height(0)));
    let root = match align_self {
        Some(align) => root.style(style().align_self(align)),
        None => root,
    };
    if scrollable.get() {
        node().child(scroll(root).axes(ScrollAxes::Vertical))
    } else {
        root
    }
}
