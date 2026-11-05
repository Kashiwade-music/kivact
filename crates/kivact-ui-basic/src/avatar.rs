//! Role: Lyra avatar, image clipping and facepile member recipes.
use crate::Tokens;
use kivact::style::{Align, Direction, Justify, Position, Style, Theme, pct, style};
use kivact::ui::{AvatarShape, AvatarState, ControlSize, Look};
fn side(size: ControlSize) -> f64 {
    match size {
        ControlSize::ExtraSmall => 20.0,
        ControlSize::Small => 24.0,
        ControlSize::Medium => 32.0,
        ControlSize::Large => 40.0,
    }
}
fn radius(t: &Theme, shape: AvatarShape) -> f64 {
    match shape {
        AvatarShape::Circle => 999.0,
        AvatarShape::Rounded => t.radius().md,
        AvatarShape::Square => 0.0,
    }
}
pub(crate) fn avatar(t: &Theme, s: &Look<AvatarState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .position(Position::Relative)
        .size(side(s.size), side(s.size))
        .shrink(0)
        .radius(radius(t, s.shape))
        .background(p.muted)
        .color(p.muted_foreground)
        .font_size(if s.size == ControlSize::Small {
            12.0
        } else {
            14.0
        })
        .text_wrap(false)
        .clip(false)
        .ring(1, p.border)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}
pub(crate) fn image(t: &Theme, s: &Look<AvatarState>) -> Style {
    style()
        .size(pct(100), pct(100))
        .radius(radius(t, s.shape))
        .clip(true)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}
pub(crate) fn group(_: &Theme, _: &Look<()>) -> Style {
    style().direction(Direction::Row).align_items(Align::Center)
}
pub(crate) fn member(t: &Theme, _: &Look<usize>) -> Style {
    style()
        .radius(999)
        .ring(2, t.ext::<Tokens>().background)
        .shrink(0)
}
pub(crate) fn status(t: &Theme, s: &Look<AvatarState>) -> Style {
    let p = t.ext::<Tokens>();
    let dot = match s.size {
        ControlSize::ExtraSmall | ControlSize::Small => 8.0,
        ControlSize::Medium => 10.0,
        ControlSize::Large => 12.0,
    };
    style()
        .position(Position::Absolute)
        .right(0)
        .bottom(0)
        .size(dot, dot)
        .radius(999)
        .background(match s.status.unwrap_or_default() {
            kivact::interaction::Tone::Neutral | kivact::interaction::Tone::Accent => p.primary,
            kivact::interaction::Tone::Info => kivact::style::ColorToken::Accent.into(),
            kivact::interaction::Tone::Success => kivact::style::ColorToken::Success.into(),
            kivact::interaction::Tone::Warning => kivact::style::ColorToken::Warning.into(),
            kivact::interaction::Tone::Danger => p.destructive,
        })
        .color(p.primary_foreground)
        .ring(2, p.background)
        .z_index(10)
}
