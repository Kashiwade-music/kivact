//! Role: The badge recipe state, tokens, recipe table and default looks.
use crate::look_common::tone;

use kivact_core::style::{Align, ColorToken, Justify, Style, Theme, style};
use kivact_interaction::Tone;
pub(crate) fn badge(t: &Theme, s: &BadgeState) -> Style {
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

pub(crate) fn badge_label(_: &Theme, _: &BadgeState) -> Style {
    style()
}

#[derive(Clone, Debug, PartialEq)]
pub struct BadgeState {
    pub tone: Tone,
    pub input: kivact_core::custom::PartState,
    /// Whether the badge has an activation callback.
    pub actionable: bool,
    pub leading_icon: bool,
    pub trailing_icon: bool,
}
