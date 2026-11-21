//! Role: The badge recipe state, recipe table and default looks.
use crate::look_common::tone;
use crate::recipe::recipe_table;
use crate::theme::Look;

use kivact_core::style::{Align, Justify, Style, Theme, style};
use kivact_interaction::Tone;
pub(crate) fn badge(t: &Theme, s: &Look<BadgeState>) -> Style {
    style()
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(t.get(&kivact_core::style::space::SM_VAR))
        .padding_x(t.get(&kivact_core::style::space::SM_VAR))
        .min_height(20)
        .border_radius(t.get(&kivact_core::style::radius::SM_VAR))
        .background(kivact_core::style::color::SURFACE)
        .color(kivact_core::style::color::TEXT)
        .font_size(12)
        .color(tone(s.tone))
}

pub(crate) fn badge_label(_: &Theme, _: &Look<BadgeState>) -> Style {
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

recipe_table! {
    /// The recipes of a badge: its root and its label.
    BadgeTheme {
        root(root_recipe): BadgeState = badge,
        label(label_recipe): BadgeState = badge_label,
    }
}
