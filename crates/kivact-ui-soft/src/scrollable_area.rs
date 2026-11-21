//! Role: Add a named, keyboard-accessible Neutral scroll viewport.
use crate::color as c;
use crate::recipe_parts::ring;
use crate::space::Space;
use crate::stack::{ContainerInset, Padding};

use kivact::view::Element;
use kivact::view::ScrollAxes;
use kivact::view::column;
use kivact::view::node_ref;
use kivact::view::scroll;

use kivact::Prop;
use kivact::component;
use kivact::state::Value;
use kivact::state::provide;
use kivact::state::try_use_context;
use kivact::style::Color;
use kivact::style::Length;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::Role;
use kivact::view::View;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
/// Scrollable directions supported by a viewport.
pub enum ScrollAxis {
    /// Scroll horizontally.
    Inline,
    #[default]
    /// Scroll vertically; the default.
    Block,
    /// Allow both horizontal and vertical scrolling.
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

/// Named region with native scrolling and a tab stop when its content overflows.
///
/// `label` names the region; the viewport is vertical unless `axis` says otherwise.
#[component]
pub fn scrollable_area(
    label: Value<String>,
    child: View,
    #[prop(default = ScrollAxis::Block)] axis: Value<ScrollAxis>,
    padding: Option<Space>,
    padding_x: Option<Space>,
    padding_y: Option<Space>,
    padding_start: Option<Space>,
    padding_end: Option<Space>,
    padding_top: Option<Space>,
    padding_bottom: Option<Space>,
    #[prop(default = false)] full_bleed: Value<bool>,
    #[prop(default = false)] bleed_top: Value<bool>,
    #[prop(default = false)] bleed_bottom: Value<bool>,
    force_focusable: Option<Value<bool>>,
) -> impl Element {
    let padding = Padding {
        all: padding,
        x: padding_x,
        y: padding_y,
        start: padding_start,
        end: padding_end,
        top: padding_top,
        bottom: padding_bottom,
    };
    let viewport = node_ref();
    let content = node_ref();
    let state = viewport.scroll_state();
    let inset = padding.resolve(0.0);
    let parent = try_use_context::<ContainerInset>().unwrap_or_default();
    let inner = column(provide(inset, move || child))
        .node_ref(content)
        .with_node(move |node| {
            node.base_style(move || {
                let base = inset.apply(
                    style()
                        .min_width(pct(100))
                        .min_height(pct(100))
                        .flex_shrink(0),
                );
                if axis.get() == ScrollAxis::Block {
                    base
                } else {
                    base.width(Length::MaxContent)
                }
            })
        });
    scroll(inner)
        .axes(axis.get().native())
        .name(label)
        .with_node(move |node| node.role(Role::Region))
        .with_node(move |node| {
            node.base_style(move || {
                let mut base = style()
                    .min_height(0)
                    .min_width(0)
                    .max_width(pct(100))
                    .max_height(pct(100))
                    .scrollbar_color(c::NEUTRAL, Color::TRANSPARENT)
                    .focus_visible(ring(c::ACCENT));
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
        })
        .node_ref(viewport)
        .focusable(move || {
            force_focusable.is_some_and(|force| force.get())
                || state.get() != kivact::view::ScrollState::NotScrollable
        })
}
