//! Role: Provide Astryx spacing and flex layout primitives.

use crate::tokens;
use kivact::control::{Node, NodeBuilder, node};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Direction, Justify, Length, Style, style};
use kivact::view::{IntoView, ReadingDirection, View, current_direction};

/// A public Neutral spacing step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Space {
    #[default]
    S0,
    S0_5,
    S1,
    S1_5,
    S2,
    S3,
    S4,
    S5,
    S6,
    S8,
    S10,
}

impl Space {
    /// Resolve this spacing step against the current theme.
    pub fn px(self) -> f32 {
        let s = tokens().spacing;
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
    current_direction() == ReadingDirection::Rtl
}

#[derive(Clone, Copy, Debug, Default)]
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
    pub start: f32,
    pub end: f32,
    pub top: f32,
    pub bottom: f32,
}

impl ContainerInset {
    pub(crate) fn uniform(px: f32) -> Self {
        Self {
            start: px,
            end: px,
            top: px,
            bottom: px,
        }
    }

    pub(crate) fn sides(self) -> (f32, f32) {
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
    pub(crate) fn resolve(self, default: f32) -> ContainerInset {
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
        pub fn padding(mut self, value: Space) -> Self {
            self.padding.all = Some(value);
            self
        }
        pub fn padding_x(mut self, value: Space) -> Self {
            self.padding.x = Some(value);
            self
        }
        pub fn padding_y(mut self, value: Space) -> Self {
            self.padding.y = Some(value);
            self
        }
        pub fn padding_start(mut self, value: Space) -> Self {
            self.padding.start = Some(value);
            self
        }
        pub fn padding_end(mut self, value: Space) -> Self {
            self.padding.end = Some(value);
            self
        }
        pub fn padding_top(mut self, value: Space) -> Self {
            self.padding.top = Some(value);
            self
        }
        pub fn padding_bottom(mut self, value: Space) -> Self {
            self.padding.bottom = Some(value);
            self
        }
    };
}
pub(crate) use padding_options;

/// A direction-aware flex container.
pub struct Stack {
    root: Node,
    direction: Direction,
    gap: Space,
    padding: Padding,
    align: Align,
    justify: Justify,
    wrap: bool,
}

pub fn stack(children: impl IntoView) -> Stack {
    vstack(children)
}

pub fn hstack(children: impl IntoView) -> Stack {
    vstack(children).direction(Direction::Row)
}

pub fn vstack(children: impl IntoView) -> Stack {
    Stack {
        root: node().child(children),
        direction: Direction::Column,
        gap: Space::S0,
        padding: Padding::default(),
        align: Align::Stretch,
        justify: Justify::Start,
        wrap: false,
    }
}

/// A horizontal stack centered on both axes.
pub fn center(children: impl IntoView) -> Stack {
    hstack(children)
        .align(Align::Center)
        .justify(Justify::Center)
}

impl Stack {
    padding_options!();
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
    pub fn gap(mut self, gap: Space) -> Self {
        self.gap = gap;
        self
    }
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.root = self.root.style(style().width(width));
        self
    }
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.root = self.root.style(style().height(height));
        self
    }
}

impl NodeBuilder for Stack {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Stack {
    fn into_view(self) -> View {
        let Self {
            root,
            direction,
            gap,
            padding,
            align,
            justify,
            wrap,
        } = self;
        root.base_style(move || {
            let direction = match (direction, rtl()) {
                (Direction::Row, true) => Direction::RowReverse,
                (Direction::RowReverse, true) => Direction::Row,
                _ => direction,
            };
            padding.resolve(0.0).apply(
                style()
                    .direction(direction)
                    .gap(gap.px())
                    .align_items(align)
                    .justify_content(justify)
                    .wrap(wrap),
            )
        })
        .into_view()
    }
}

/// Flex sizing for a stack item.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StackItemSize {
    #[default]
    Auto,
    Fill,
    Static,
}

pub struct StackItem {
    root: Node,
    size: StackItemSize,
}

pub fn stack_item(children: impl IntoView) -> StackItem {
    StackItem {
        root: node().child(children),
        size: StackItemSize::Auto,
    }
}

impl StackItem {
    pub fn size(mut self, size: StackItemSize) -> Self {
        self.size = size;
        self
    }
    pub fn align_self(mut self, align: Align) -> Self {
        self.root = self.root.style(style().align_self(align));
        self
    }
}

impl NodeBuilder for StackItem {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for StackItem {
    fn into_view(self) -> View {
        let sizing = match self.size {
            StackItemSize::Auto => style().grow(0).shrink(1),
            StackItemSize::Fill => style().grow(1).shrink(1).min_width(0).min_height(0),
            StackItemSize::Static => style().grow(0).shrink(0),
        };
        self.root.base_style(sizing).into_view()
    }
}
