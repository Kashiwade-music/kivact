//! Role: Draw presence indicators and report their labels to the containing avatar.
use crate::color as c;
use crate::metrics::{StatusMetrics, status_metrics};
use crate::radius as r;
use crate::variants::AvatarSize;

use kivact::component;
use kivact::state::Value;
use kivact::style::Align;
use kivact::style::Color;
use kivact::style::FlexDirection;
use kivact::style::Justify;
use kivact::style::style;
use kivact::view::Element;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::node;

/// The meaning and appearance of a presence indicator.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AvatarStatus {
    /// Success-colored presence marker, the default.
    #[default]
    Success,
    /// Neutral presence marker.
    Neutral,
    /// Error-colored presence marker.
    Error,
    /// Explicit background and glyph colors.
    Custom {
        /// Background color of the marker.
        fill: Color,
        /// Foreground color of its glyph.
        ink: Color,
    },
}

/// A small presence indicator, usable with or without an avatar.
///
/// `avatar_size` matches the containing standard avatar diameter. `label` is
/// the text exposed to assistive technology, and `icon` replaces the default
/// glyph at medium and large sizes.
#[component]
pub fn avatar_status_dot(
    status: AvatarStatus,
    #[prop(default = AvatarSize::Md)] avatar_size: AvatarSize,
    label: Option<Value<String>>,
    icon: Option<View>,
) -> impl Element {
    let StatusMetrics {
        side,
        border,
        stroke,
        icon: icon_side,
    } = status_metrics(avatar_size.side());
    let mut root = node();
    if let Some(label) = label {
        root = root.role(Role::Image).name(label);
    }
    let field = side - 2.0 * border;
    let glyph = if let Some(icon) = icon.filter(|_| icon_side > 0.0) {
        Some(
            node()
                .style(
                    style()
                        .size(icon_side, icon_side)
                        .align_items(Align::Center)
                        .justify_content(Justify::Center),
                )
                .child(icon),
        )
    } else {
        match status {
            AvatarStatus::Neutral => Some(node().style(move || {
                style()
                    .size(field, field)
                    .border_radius(r::RADIUS_FULL)
                    .border(stroke, c::TEXT_SECONDARY)
            })),
            AvatarStatus::Error => Some(node().style(move || {
                style()
                    .size(field * 0.75, stroke)
                    .border_radius(r::RADIUS_FULL)
                    .background(c::ON_ERROR)
            })),
            _ => None,
        }
    };
    root.with_node(move |node| {
        node.base_style(move || {
            let (fill, ink) = match status {
                AvatarStatus::Success => (c::SUCCESS, c::ON_SUCCESS),
                AvatarStatus::Neutral => (c::BG_SURFACE, c::TEXT_SECONDARY),
                AvatarStatus::Error => (c::ERROR, c::ON_ERROR),
                AvatarStatus::Custom { fill, ink } => (fill, ink),
            };
            style()
                .size(side, side)
                .flex_shrink(0)
                .border_radius(r::RADIUS_FULL)
                .border(border, c::BG_SURFACE)
                .background(fill)
                .color(ink)
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
        })
    })
    .child(glyph)
}
