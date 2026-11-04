//! Role: Add a named, keyboard-accessible Neutral scroll viewport.
use crate::stack::{ContainerInset, Padding, Space, padding_options, rtl};
use crate::tokens;
use kivact::state::IntoValue;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::ScrollAxes;
use kivact::view::column;
use kivact::view::node;
use kivact::view::scroll;

use kivact::state::IntoText;
use kivact::state::Value;
use kivact::state::memo;
use kivact::state::provide;
use kivact::state::try_use_context;
use kivact::style::Color;
use kivact::style::Length;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::Handled;
use kivact::view::IntoView;
use kivact::view::Key;
use kivact::view::KeyEvent;
use kivact::view::NodeRef;
use kivact::view::Role;
use kivact::view::ScrollBehavior;
use kivact::view::View;
use kivact::view::node_ref;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollAxis {
    Inline,
    #[default]
    Block,
    Both,
}

impl ScrollAxis {
    fn native(self) -> ScrollAxes {
        match self {
            Self::Inline => ScrollAxes::Horizontal,
            Self::Block => ScrollAxes::Vertical,
            Self::Both => ScrollAxes::Both,
        }
    }
}

pub struct ScrollableArea {
    root: Node,
    child: View,
    axis: Value<ScrollAxis>,
    padding: Padding,
    full_bleed: Value<bool>,
    bleed_top: Value<bool>,
    bleed_bottom: Value<bool>,
    force_focusable: Option<Value<bool>>,
}

pub fn scrollable_area(label: impl IntoText, child: impl IntoView) -> ScrollableArea {
    ScrollableArea {
        root: node()
            .name(label)
            .with_node(move |node| node.role(Role::Region)),
        child: child.into_view(),
        axis: (ScrollAxis::Block).into_value(),
        padding: Padding::default(),
        full_bleed: (false).into_value(),
        bleed_top: (false).into_value(),
        bleed_bottom: (false).into_value(),
        force_focusable: None,
    }
}

impl ScrollableArea {
    padding_options!();
    pub(crate) fn force_focusable(mut self, value: Value<bool>) -> Self {
        self.force_focusable = Some(value);
        self
    }
    pub fn axis(mut self, axis: impl IntoValue<ScrollAxis>) -> Self {
        self.axis = axis.into_value();
        self
    }
    pub fn full_bleed(mut self, full_bleed: impl IntoValue<bool>) -> Self {
        self.full_bleed = full_bleed.into_value();
        self
    }
    pub fn bleed_top(mut self, bleed: impl IntoValue<bool>) -> Self {
        self.bleed_top = bleed.into_value();
        self
    }
    pub fn bleed_bottom(mut self, bleed: impl IntoValue<bool>) -> Self {
        self.bleed_bottom = bleed.into_value();
        self
    }
}

impl NodeBuilder for ScrollableArea {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for ScrollableArea {
    fn into_view(self) -> View {
        let Self {
            root,
            child,
            axis,
            padding,
            full_bleed,
            bleed_top,
            bleed_bottom,
            force_focusable,
        } = self;
        provide((), move || {
            let viewport = node_ref();
            let content = node_ref();
            let state = use_scroll_state(viewport, content, axis.get());
            let inset = padding.resolve(0.0);
            let parent = try_use_context::<ContainerInset>()
                .map(|value| value.get())
                .unwrap_or_default();
            let inner = column(provide(inset, move || child))
                .node_ref(content)
                .with_node(move |node| {
                    node.base_style(move || {
                        let base =
                            inset.apply(style().min_width(pct(100)).min_height(pct(100)).shrink(0));
                        if axis.get() == ScrollAxis::Block {
                            base
                        } else {
                            base.width(Length::MaxContent)
                        }
                    })
                });
            let focused = viewport.focused();
            let root = root.with_node(move |node| {
                node.base_style(move || {
                    let t = tokens();
                    let mut base = style()
                        .min_height(0)
                        .min_width(0)
                        .max_width(pct(100))
                        .max_height(pct(100))
                        .scrollbar_color(t.colors.neutral, Color::TRANSPARENT)
                        .focus_visible(crate::recipe_style::ring(&t, t.colors.accent));
                    if full_bleed.get() {
                        let (left, right) = parent.sides();
                        base = base
                            .margin_left(-left)
                            .margin_right(-right)
                            .max_width(pct(100) + left + right);
                        if bleed_top.get() {
                            base = base.margin_top(-parent.top);
                        }
                        if bleed_bottom.get() {
                            base = base.margin_bottom(-parent.bottom);
                        }
                    }
                    base
                })
            });
            let mut viewport_node = scroll(inner).axes(axis.get().native());
            *viewport_node.node_mut() = root;
            viewport_node
                .node_ref(viewport)
                .focusable(move || {
                    force_focusable.is_some_and(|force| force.get())
                        || state.inline.is_scrollable.get()
                        || state.block.is_scrollable.get()
                })
                .on_key(move |event| {
                    if focused.get() {
                        scroll_key(viewport, state, event)
                    } else {
                        Handled::No
                    }
                })
        })
    }
}

/// Observed state of one logical scroll axis.
#[derive(Clone, Copy)]
pub struct AxisScroll {
    pub is_scrollable: Value<bool>,
    pub at_start: Value<bool>,
    pub at_end: Value<bool>,
}

/// Observed geometry of a viewport and its content box.
#[derive(Clone, Copy)]
pub struct ScrollState {
    pub inline: AxisScroll,
    pub block: AxisScroll,
}

pub fn use_scroll_state(viewport: NodeRef, content: NodeRef, axis: ScrollAxis) -> ScrollState {
    let observe = |inline: bool, enabled: bool| {
        let extent = move || {
            let Some(outer) = viewport.rect().get() else {
                return 0.0;
            };
            let Some(inner) = content.rect().get() else {
                return 0.0;
            };
            if inline {
                (inner.width - outer.width).max(0.0)
            } else {
                (inner.height - outer.height).max(0.0)
            }
        };
        let is_scrollable = memo(move || enabled && extent() > 1.0);
        let at_boundary = move |start: bool| {
            if !enabled || extent() <= 1.0 {
                return true;
            }
            let offset = viewport.scroll_offset().get();
            let value = if inline { offset.x } else { offset.y };
            let physical_start = if inline && rtl() { !start } else { start };
            if physical_start {
                value <= 1.0
            } else {
                value >= extent() - 1.0
            }
        };
        AxisScroll {
            is_scrollable,
            at_start: memo(move || at_boundary(true)),
            at_end: memo(move || at_boundary(false)),
        }
    };
    ScrollState {
        inline: observe(true, axis != ScrollAxis::Block),
        block: observe(false, axis != ScrollAxis::Inline),
    }
}

#[derive(Clone, Copy)]
pub struct ScrollOverflow {
    pub start: Value<bool>,
    pub end: Value<bool>,
    pub any: Value<bool>,
}

pub fn use_scroll_overflow(viewport: NodeRef, content: NodeRef) -> ScrollOverflow {
    let axis = use_scroll_state(viewport, content, ScrollAxis::Inline).inline;
    ScrollOverflow {
        start: memo(move || !axis.at_start.get()),
        end: memo(move || !axis.at_end.get()),
        any: axis.is_scrollable,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OverflowOptions {
    pub gap: f64,
    pub min_visible_items: usize,
    pub collapse_from_start: bool,
}

pub fn use_overflow(
    widths: Value<Vec<f64>>,
    available: Value<f64>,
    indicator: Value<f64>,
    options: OverflowOptions,
) -> Value<usize> {
    memo(move || {
        let widths = widths.get();
        let available = available.get();
        let indicator = indicator.get();
        let total = widths.len();
        let mut taken = 0.0;
        for index in 0..total {
            let width = widths[if options.collapse_from_start {
                total - index - 1
            } else {
                index
            }];
            taken += width + if index == 0 { 0.0 } else { options.gap };
            let reserve = if index + 1 < total {
                options.gap + indicator
            } else {
                0.0
            };
            if index >= options.min_visible_items && taken + reserve > available {
                return index;
            }
        }
        total
    })
}

fn scroll_key(viewport: NodeRef, state: ScrollState, event: KeyEvent) -> Handled {
    use kivact::geometry::Point;
    let Some(rect) = viewport.rect().get() else {
        return Handled::No;
    };
    let inline = state.inline.is_scrollable.get();
    let block = state.block.is_scrollable.get();
    if event.modifiers().ctrl || event.modifiers().alt || event.modifiers().meta {
        return Handled::No;
    }
    let (x, y) = match event.key() {
        Key::ArrowLeft if inline => (-40.0, 0.0),
        Key::ArrowRight if inline => (40.0, 0.0),
        Key::ArrowUp if block => (0.0, -40.0),
        Key::ArrowDown if block => (0.0, 40.0),
        Key::PageUp | Key::PageDown | Key::Space | Key::Home | Key::End if inline || block => {
            let backward = matches!(event.key(), Key::PageUp | Key::Home)
                || event.key() == Key::Space && event.modifiers().shift;
            let sign = if backward { -1.0 } else { 1.0 };
            let distance = if matches!(event.key(), Key::Home | Key::End) {
                f64::MAX
            } else if block {
                rect.height
            } else {
                rect.width
            };
            if block {
                (0.0, sign * distance)
            } else {
                (sign * distance * if rtl() { -1.0 } else { 1.0 }, 0.0)
            }
        }
        _ => return Handled::No,
    };
    viewport.scroll_by(Point { x, y }, ScrollBehavior::Instant);
    Handled::Yes
}

kivact::state::constant_value!(ScrollAxis);
