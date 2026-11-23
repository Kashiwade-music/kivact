//! Role: Layer actions or labels within the bounds of their media.
use crate::color as c;
use crate::motion::FAST;
use crate::recipe_parts::{ring, tween};

use kivact::Prop;
use kivact::component;
use kivact::state::Value;
use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::Overflow;
use kivact::style::Position;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::OverlayAlign;
use kivact::view::View;
use kivact::view::column;
use kivact::view::node_ref;

/// Visibility trigger for media overlay content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum ShowOn {
    /// Keep the overlay visible, the default.
    #[default]
    Always,
    /// Reveal while the base is hovered or contains keyboard focus.
    Hover,
    /// Reveal while the base contains keyboard focus.
    Focus,
}

/// Background treatment behind overlay content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum OverlayScrim {
    /// Dark scrim, the default.
    #[default]
    Dark,
    /// Light scrim.
    Light,
    /// No scrim.
    None,
}

/// Placement of content inside the media bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum OverlayPosition {
    /// Fill the media bounds, the default.
    #[default]
    Fill,
    /// Place content along the bottom edge.
    Bottom,
    /// Place content along the top edge.
    Top,
}

/// Media-bound `content` layered over `base`, shown by `show_on` or `open`.
///
/// `open` overrides the automatic visibility of `show_on` with a read-only input.
#[component]
pub fn media_overlay(
    content: View,
    base: View,
    #[prop(default = ShowOn::Always)] show_on: ShowOn,
    open: Option<Value<bool>>,
    #[prop(default = OverlayScrim::Dark)] scrim: OverlayScrim,
    #[prop(default = OverlayPosition::Fill)] position: OverlayPosition,
    #[prop(default = OverlayAlign::End)] align: OverlayAlign,
) -> impl Element {
    let root = node_ref();
    let hover = root.hovered();
    let focus = root.focus_within();
    let mode = show_on;
    let visible = kivact::state::memo(move || {
        open.map_or_else(
            || match mode {
                ShowOn::Always => true,
                ShowOn::Hover => hover.get() || focus.get(),
                ShowOn::Focus => focus.get(),
            },
            |open| open.get(),
        )
    });
    let reveal_from_keyboard = open.is_none() && mode != ShowOn::Always;
    let overlay = column(content)
        .hidden(move || !visible.get())
        .style(move || {
            let background = match scrim {
                OverlayScrim::Dark => c::OVERLAY_SCRIM,
                OverlayScrim::Light => c::OVERLAY_LIGHT_SCRIM,
                OverlayScrim::None => kivact::style::Color::TRANSPARENT,
            };
            let (align_items, justify) = match align {
                OverlayAlign::Start => (Align::Start, Justify::Start),
                OverlayAlign::Center => (Align::Center, Justify::Center),
                OverlayAlign::End => (Align::End, Justify::Start),
            };
            let layer = style()
                .position(Position::Absolute)
                .left(0)
                .right(0)
                .gap(8)
                .padding(12)
                .background(background)
                .align_items(align_items)
                .justify_content(justify);
            let (layer, offset) = match position {
                OverlayPosition::Fill => (layer.top(0).bottom(0), 0),
                OverlayPosition::Top => (layer.top(0), -100),
                OverlayPosition::Bottom => (layer.bottom(0), 100),
            };
            let hidden = style().opacity(0.0).translate_y(pct(offset));
            let layer = tween(layer, FAST).enter(hidden.clone());
            if visible.get() {
                layer
            } else {
                layer.merge(&hidden)
            }
        });
    column((base, overlay))
        .node_ref(root)
        .focusable(reveal_from_keyboard)
        .style(|| {
            style()
                .overflow(Overflow::Hidden)
                .focus_visible(ring(c::FOCUS_RING))
        })
}
