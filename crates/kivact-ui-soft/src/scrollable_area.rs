//! Role: Add a named, keyboard-accessible Neutral scroll viewport.
use crate::stack::{ContainerInset, Padding, Space, padding_options};

use kivact::state::IntoValue;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::ScrollAxes;
use kivact::view::column;
use kivact::view::node;
use kivact::view::scroll;

use kivact::state::IntoText;
use kivact::state::Value;
use kivact::state::provide;
use kivact::state::try_use_context;
use kivact::style::Color;
use kivact::style::Length;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::Role;
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

// xtask-allow(state-fields): Independent NodeBuilder setup options for axis, padding, bleed, and focusability.
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
        kivact::view::component(move || {
            let viewport = node_ref();
            let content = node_ref();
            let state = viewport.scroll_state();
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
            let root = root.with_node(move |node| {
                node.base_style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
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
            viewport_node.node_ref(viewport).focusable(move || {
                force_focusable.is_some_and(|force| force.get())
                    || state.get() != kivact::view::ScrollState::NotScrollable
            })
        })
    }
}

kivact::state::constant_value!(ScrollAxis);
