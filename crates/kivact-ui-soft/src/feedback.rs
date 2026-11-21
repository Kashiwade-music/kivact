//! Role: Render application status dots and shared icon boxes.
use crate::color as c;
use crate::radius as r;

use kivact::Prop;
use kivact::component;
use kivact::motion::Animated;
use kivact::motion::Easing;
use kivact::motion::Repeat;
use kivact::motion::Transition;
use kivact::state::Value;
use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::style;
use kivact::view::Element;
use kivact::view::Node;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::node;
use kivact::view::row;

pub(crate) fn icon_box(icon: View, size: f64) -> Node {
    node()
        .style(style().size(size, size).flex_shrink(0))
        .child(icon)
}

/// Status represented by a dot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Prop)]
pub enum StatusDotVariant {
    /// Success-colored dot.
    Success,
    /// Warning-colored dot.
    Warning,
    /// Error-colored dot.
    Error,
    /// Accent-colored dot.
    Accent,
    /// Neutral-colored dot.
    Neutral,
}

/// Named noninteractive status indicator with an accessible label.
#[component]
pub fn status_dot(
    variant: StatusDotVariant,
    label: Value<String>,
    icon: Option<View>,
    #[prop(default = false)] pulsing: Value<bool>,
) -> impl Element {
    let pulse = pulsing.get().then(|| {
        Animated::from(
            1.0f64,
            0.5f64,
            Transition::tween(1000, Easing::CubicBezier(0.42, 0.0, 0.58, 1.0))
                .repeat(Repeat::Forever)
                .alternate(),
        )
    });
    row(())
        .with_node(move |node| node.role(Role::Image))
        .name(label)
        .with_node(move |node| {
            node.base_style(move || {
                let (fill, ink) = match variant {
                    StatusDotVariant::Success => (c::SUCCESS, c::ON_SUCCESS),
                    StatusDotVariant::Warning => (c::WARNING, c::ON_WARNING),
                    StatusDotVariant::Error => (c::ERROR, c::ON_ERROR),
                    StatusDotVariant::Accent => (c::ACCENT, c::ON_ACCENT),
                    StatusDotVariant::Neutral => (c::ICON_SECONDARY, c::BG_SURFACE),
                };
                let opacity = if kivact::platform::is_motion_reduced().get() {
                    1.0
                } else {
                    pulse.as_ref().map_or(1.0, Animated::get)
                };
                style()
                    .size(8, 8)
                    .flex_shrink(0)
                    .border_radius(r::RADIUS_FULL)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .background(fill)
                    .color(ink)
                    .opacity(opacity)
            })
        })
        .child(icon.map(|icon| icon_box(icon, 8.0)))
}
