//! Role: Supply theme-based default indicator geometry and drawing.
use crate::{
    AvatarShape, AvatarState, BadgeState, ControlSize, SkeletonShape, SkeletonState, SpinnerState,
};
use kivact_core::draw::{LineCap, Path, Stroke};
use kivact_core::geometry::Point;
use kivact_core::style::{Align, ColorToken, Justify, Style, Theme, pct, style};
pub(super) fn side(size: ControlSize) -> f64 {
    match size {
        ControlSize::ExtraSmall => 20.0,
        ControlSize::Small => 24.0,
        ControlSize::Medium => 36.0,
        ControlSize::Large => 48.0,
    }
}
pub(super) fn badge(t: &Theme, s: &BadgeState) -> Style {
    style()
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(t.space().sm)
        .padding_x(t.space().sm)
        .min_height(20)
        .radius(t.radius().sm)
        .background(ColorToken::Surface)
        .color(ColorToken::Text)
        .font_size(12)
        .color(tone(s.tone))
}
pub(super) fn avatar(t: &Theme, s: &AvatarState) -> Style {
    style()
        .size(side(s.size), side(s.size))
        .radius(match s.shape {
            AvatarShape::Circle => 999.0,
            AvatarShape::Rounded => t.radius().md,
            AvatarShape::Square => 0.0,
        })
        .background(ColorToken::Surface)
        .color(ColorToken::Text)
        .clip(true)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}
pub(super) fn skeleton(t: &Theme, s: &SkeletonState) -> Style {
    let tokens = t.ext::<crate::UiTokens>();
    let base = style()
        .min_height(tokens.indicators.line_height)
        .width(pct(100))
        .background(ColorToken::Border)
        .opacity(s.pulse);
    match s.shape {
        SkeletonShape::Circle => base.size(36, 36).radius(999),
        SkeletonShape::Rounded => base.radius(t.radius().md),
        SkeletonShape::Rect => base.radius(0),
        SkeletonShape::Text => base.radius(t.radius().sm),
    }
}
pub(super) fn spinner(_: &Theme, s: &SpinnerState) -> Style {
    let side = side(s.size) * 0.6;
    let turn = s.turn;
    style().size(side, side).shrink(0).paint(move |d, size| {
        let center = Point::new(size.width / 2.0, size.height / 2.0);
        let arc = Path::new().arc(
            center,
            size.width.min(size.height) / 2.0 - 2.0,
            turn,
            turn + 270.0,
            false,
        );
        d.stroke_path(
            &arc,
            &Stroke::new(2.0).cap(LineCap::Round),
            ColorToken::Text,
        );
    })
}
pub(super) fn kbd(_: &Theme, _: &()) -> Style {
    style().align_items(Align::Center).gap(4).shrink(0)
}

pub(super) fn kbd_key(t: &Theme, _: &crate::KbdKeyState) -> Style {
    style()
        .min_height(20)
        .min_width(20)
        .padding_x(t.space().sm)
        .radius(t.radius().sm)
        .background(ColorToken::Surface)
        .border_bottom(2, ColorToken::Border)
        .color(ColorToken::Text)
        .font_size(12)
        .text_wrap(false)
}

pub(super) fn avatar_status(_: &Theme, s: &AvatarState) -> Style {
    style()
        .position(kivact_core::style::Position::Absolute)
        .right(0)
        .bottom(0)
        .size(8, 8)
        .radius(999)
        .background(tone(s.status.unwrap_or_default()))
}
pub(super) fn avatar_group(_: &Theme, _: &()) -> Style {
    style().align_items(Align::Center)
}

fn tone(tone: kivact_interaction::Tone) -> ColorToken {
    match tone {
        kivact_interaction::Tone::Neutral => ColorToken::TextMuted,
        kivact_interaction::Tone::Info | kivact_interaction::Tone::Accent => ColorToken::Accent,
        kivact_interaction::Tone::Success => ColorToken::Success,
        kivact_interaction::Tone::Warning => ColorToken::Warning,
        kivact_interaction::Tone::Danger => ColorToken::Danger,
    }
}

pub(super) fn avatar_group_overflow(_: &Theme, _: &usize) -> Style {
    style()
}
pub(super) fn avatar_group_member(_: &Theme, _: &usize) -> Style {
    style()
}
