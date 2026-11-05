//! Role: Style canonical avatars and supply an optional person silhouette.
use crate::AvatarSize;
use kivact::state::{IntoValue, Value};
use kivact::style::{Align, Direction, Justify, Position, Theme, pct, style};
use kivact::ui::{AvatarShape, AvatarState, ControlSize, Look};
use kivact::view::{Asset, image};
const PERSON: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/></svg>"#;

/// Shared visual frame of actual canonical group members.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AvatarGroupAppearance {
    /// Reactive diameter shared by the group members.
    pub size: Value<AvatarSize>,
    /// Reactive silhouette shared by the group members.
    pub shape: Value<AvatarShape>,
}
impl AvatarGroupAppearance {
    /// Combine read-only size and silhouette inputs for a canonical avatar group.
    pub fn new(size: impl IntoValue<AvatarSize>, shape: impl IntoValue<AvatarShape>) -> Self {
        Self {
            size: size.into_value(),
            shape: shape.into_value(),
        }
    }
}
impl kivact::ui::Variant for AvatarGroupAppearance {}
/// A reactive diameter for standard avatars; shape remains a canonical option.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AvatarAppearance {
    size: Value<AvatarSize>,
}
impl AvatarAppearance {
    /// Use a read-only reactive diameter with a canonical avatar.
    pub fn new(size: impl IntoValue<AvatarSize>) -> Self {
        Self {
            size: size.into_value(),
        }
    }
}
impl kivact::ui::Variant for AvatarAppearance {}
fn side(s: &Look<AvatarState>) -> f64 {
    s.variant::<AvatarAppearance>().map_or_else(
        || {
            s.variant::<AvatarSize>().copied().map_or_else(
                || match s.size {
                    ControlSize::ExtraSmall => 20.0,
                    ControlSize::Small => 24.0,
                    ControlSize::Medium => 36.0,
                    ControlSize::Large => 48.0,
                },
                AvatarSize::side,
            )
        },
        |v| v.size.get().side(),
    )
}
fn radius(s: &Look<AvatarState>, t: &crate::Tokens) -> f64 {
    match s.shape {
        AvatarShape::Circle => t.radius.full,
        AvatarShape::Rounded => t.radius.element,
        AvatarShape::Square => t.radius.none,
    }
}
pub(crate) fn root(theme: &Theme, s: &Look<AvatarState>) -> kivact::style::Style {
    let t = theme.ext::<crate::Tokens>();
    let side = side(s);
    style()
        .size(side, side)
        .position(Position::Relative)
        .shrink(0)
        .radius(radius(s, &t))
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .direction(Direction::Row)
        .font_size(side * 0.4)
        .line_height(1.0)
        .weight(t.typography.weight_medium)
        .text_wrap(false)
        .color(t.colors.text_secondary)
        .focus_visible(crate::recipe_style::ring(&t, t.colors.focus_ring))
}
pub(crate) fn picture(theme: &Theme, s: &Look<AvatarState>) -> kivact::style::Style {
    let t = theme.ext::<crate::Tokens>();
    style()
        .size(pct(100), pct(100))
        .radius(radius(s, &t))
        .clip(true)
        .background(t.colors.neutral)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}
pub(crate) fn status(theme: &Theme, s: &Look<AvatarState>) -> kivact::style::Style {
    let t = theme.ext::<crate::Tokens>();
    let metric = crate::avatar_status_dot::status_metrics(side(s));
    let fill = match s.status.unwrap_or_default() {
        kivact::interaction::Tone::Neutral => t.colors.bg_surface,
        kivact::interaction::Tone::Info | kivact::interaction::Tone::Accent => t.colors.accent,
        kivact::interaction::Tone::Success => t.colors.success,
        kivact::interaction::Tone::Warning => t.colors.warning,
        kivact::interaction::Tone::Danger => t.colors.error,
    };
    style()
        .position(Position::Absolute)
        .right(0)
        .bottom(0)
        .size(metric.side, metric.side)
        .radius(t.radius.full)
        .border(metric.border, t.colors.bg_surface)
        .background(fill)
}
/// Pass this decorative silhouette to standard `avatar(...).fallback_content(...)`.
pub fn avatar_silhouette() -> impl kivact::view::IntoView {
    image(Asset::from(PERSON))
        .monochrome(true)
        .hit_testable(false)
        .style(style().size(pct(60), pct(60)))
}
pub(crate) fn group(_: &Theme, _: &Look<()>) -> kivact::style::Style {
    style()
        .align_items(Align::Center)
        .direction(if crate::stack::rtl() {
            Direction::RowReverse
        } else {
            Direction::Row
        })
}
pub(crate) fn member(theme: &Theme, s: &Look<usize>) -> kivact::style::Style {
    let t = theme.ext::<crate::Tokens>();
    let shape = s
        .variant::<AvatarGroupAppearance>()
        .map_or(AvatarShape::Circle, |v| v.shape.get());
    let radius = match shape {
        AvatarShape::Circle => t.radius.full,
        AvatarShape::Rounded => t.radius.element,
        AvatarShape::Square => t.radius.none,
    };
    style()
        .radius(radius)
        .border(2, t.colors.bg_surface)
        .background(t.colors.bg_surface)
}
pub(crate) fn overflow(theme: &Theme, s: &Look<usize>) -> kivact::style::Style {
    let t = theme.ext::<crate::Tokens>();
    let size = s
        .variant::<AvatarGroupAppearance>()
        .map_or(36.0, |v| v.size.get().side());
    let shape = s
        .variant::<AvatarGroupAppearance>()
        .map_or(AvatarShape::Circle, |v| v.shape.get());
    let radius = match shape {
        AvatarShape::Circle => t.radius.full,
        AvatarShape::Rounded => t.radius.element,
        AvatarShape::Square => t.radius.none,
    };
    style()
        .size(size, size)
        .radius(radius)
        .border(2, t.colors.bg_surface)
        .background(t.colors.neutral)
        .color(t.colors.text_secondary)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .font_size(size * 0.4)
        .line_height(1.0)
        .weight(t.typography.weight_medium)
}

pub(crate) fn status_content(_: &Theme, _: &Look<AvatarState>) -> kivact::style::Style {
    let base = style().position(Position::Absolute).bottom(-2);
    if crate::stack::rtl() {
        base.left(-2)
    } else {
        base.right(-2)
    }
}
