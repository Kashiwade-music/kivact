//! Role: The avatar recipe state, tokens, recipe table and default looks.
use crate::look_common::{side, tone};

use crate::{AvatarShape, ControlSize};
use kivact_core::style::{Align, ColorToken, Justify, Style, Theme, pct, style};
use kivact_interaction::Tone;
pub(crate) fn avatar(t: &Theme, s: &AvatarState) -> Style {
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

pub(crate) fn avatar_image(t: &Theme, s: &AvatarState) -> Style {
    style()
        .size(pct(100), pct(100))
        .clip(true)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .radius(match s.shape {
            AvatarShape::Circle => 999.0,
            AvatarShape::Rounded => t.radius().md,
            AvatarShape::Square => 0.0,
        })
}

pub(crate) fn avatar_status(_: &Theme, s: &AvatarState) -> Style {
    style()
        .position(kivact_core::style::Position::Absolute)
        .right(0)
        .bottom(0)
        .size(8, 8)
        .radius(999)
        .background(tone(s.status.unwrap_or_default()))
}

pub(crate) fn avatar_status_content(_: &Theme, _: &AvatarState) -> Style {
    let base = style()
        .position(kivact_core::style::Position::Absolute)
        .bottom(0);
    if kivact_core::view::use_direction().get() == kivact_core::view::ReadingDirection::Rtl {
        base.left(0)
    } else {
        base.right(0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AvatarState {
    pub size: ControlSize,
    pub shape: AvatarShape,
    pub status: Option<Tone>,
}

/// A theme's avatar defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AvatarTokens {
    /// Whether an avatar shows its name in a tooltip when the caller leaves it to the theme.
    pub name_tooltip: bool,
}
