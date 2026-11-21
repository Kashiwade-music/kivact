//! Role: Style Astryx Neutral's avatars and avatar groups: the frame, the picture, the presence marker and the overflow count.
use crate::color as c;
use crate::metrics::status_metrics;
use crate::radius as r;
use crate::recipe_parts::ring;
use crate::type_scale as t;
use crate::variants::{AvatarGroupShape, AvatarSize};
use kivact::interaction::Tone;
use kivact::style::{Align, FlexDirection, Justify, Length, Position, Style, Theme, pct, style};
use kivact::ui::theme::{AvatarGroupTheme, AvatarState, AvatarTheme, Look};
use kivact::ui::{AvatarShape, ControlSize};
use kivact::view::{ReadingDirection, use_direction};

/// The diameter of an avatar: its variant's size, or the control size's default.
fn side(look: &Look<AvatarState>) -> f64 {
    look.variant::<AvatarSize>().map_or_else(
        || match look.size {
            ControlSize::ExtraSmall => 20.0,
            ControlSize::Small => 24.0,
            ControlSize::Medium => 36.0,
            ControlSize::Large => 48.0,
        },
        AvatarSize::side,
    )
}

/// The corner radius of an avatar silhouette.
fn shape_radius(shape: AvatarShape) -> Length {
    match shape {
        AvatarShape::Circle => r::RADIUS_FULL,
        AvatarShape::Rounded => r::RADIUS_ELEMENT,
        AvatarShape::Square => r::RADIUS_NONE,
    }
}

/// The root: a centered box of the avatar's diameter, focusable with a ring.
fn root(_theme: &Theme, look: &Look<AvatarState>, _base: Style) -> Style {
    let side = side(look);
    style()
        .size(side, side)
        .position(Position::Relative)
        .flex_shrink(0.0)
        .border_radius(shape_radius(look.shape))
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .flex_direction(FlexDirection::Row)
        .font_size(side * 0.4)
        .line_height(1.0)
        .font_weight(t::WEIGHT_MEDIUM)
        .text_wrap(false)
        .color(c::TEXT_SECONDARY)
        .focus_visible(ring(c::FOCUS_RING))
}

/// The picture: fills the avatar in its silhouette, over the neutral fill.
fn picture(_theme: &Theme, look: &Look<AvatarState>, _base: Style) -> Style {
    style()
        .size(pct(100), pct(100))
        .border_radius(shape_radius(look.shape))
        .clip(true)
        .background(c::NEUTRAL)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

/// The presence marker: a small dot at the avatar's corner, in its status color.
fn status(_theme: &Theme, look: &Look<AvatarState>, _base: Style) -> Style {
    let metric = status_metrics(side(look));
    let fill = match look.status.unwrap_or_default() {
        Tone::Neutral => c::BG_SURFACE,
        Tone::Info | Tone::Accent => c::ACCENT,
        Tone::Success => c::SUCCESS,
        Tone::Warning => c::WARNING,
        Tone::Danger => c::ERROR,
    };
    style()
        .position(Position::Absolute)
        .right(0.0)
        .bottom(0.0)
        .size(metric.side, metric.side)
        .border_radius(r::RADIUS_FULL)
        .border(metric.border, c::BG_SURFACE)
        .background(fill)
}

/// The status content: pushed out past the avatar's bottom corner, on the reading side.
fn status_content(_theme: &Theme, _look: &Look<AvatarState>, _base: Style) -> Style {
    let base = style().position(Position::Absolute).bottom(-2.0);
    if use_direction() == ReadingDirection::Rtl {
        base.left(-2.0)
    } else {
        base.right(-2.0)
    }
}

/// The avatar group: a row of overlapping members, reversed in RTL.
fn group(_theme: &Theme, _look: &Look<()>, _base: Style) -> Style {
    style()
        .align_items(Align::Center)
        .flex_direction(if use_direction() == ReadingDirection::Rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
}

/// A group member: a ring in the surface color, in the group's silhouette.
fn member(_theme: &Theme, look: &Look<usize>, _base: Style) -> Style {
    let shape = look
        .variant::<AvatarGroupShape>()
        .map_or(AvatarShape::Circle, |shape| shape.0);
    style()
        .border_radius(shape_radius(shape))
        .border(2.0, c::BG_SURFACE)
        .background(c::BG_SURFACE)
}

/// The overflow count: a neutral avatar that holds the number of hidden members.
fn overflow(_theme: &Theme, look: &Look<usize>, _base: Style) -> Style {
    let size = look.variant::<AvatarSize>().map_or(36.0, AvatarSize::side);
    let shape = look
        .variant::<AvatarGroupShape>()
        .map_or(AvatarShape::Circle, |shape| shape.0);
    style()
        .size(size, size)
        .border_radius(shape_radius(shape))
        .border(2.0, c::BG_SURFACE)
        .background(c::NEUTRAL)
        .color(c::TEXT_SECONDARY)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .font_size(size * 0.4)
        .line_height(1.0)
        .font_weight(t::WEIGHT_MEDIUM)
}

/// Install the kit's recipes of the avatar and the avatar group.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme.with(
        AvatarTheme::default()
            .root(root)
            .image(picture)
            .status(status)
            .status_content(status_content),
    );
    theme.with(
        AvatarGroupTheme::default()
            .root(group)
            .member(member)
            .overflow(overflow),
    )
}
