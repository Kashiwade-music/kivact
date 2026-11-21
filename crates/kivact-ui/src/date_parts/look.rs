//! Role: The date parts recipe state, tokens, recipe table and default looks.

use crate::tokens::UiTokens;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, FlexDirection, Style, Theme, style};
pub(crate) fn input(t: &Theme, _: &PartState) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(t.get(&kivact_core::style::space::XS_VAR))
}

pub(crate) fn wheel(t: &Theme, _: &PartState) -> Style {
    style()
        .width(t.ext::<UiTokens>().date_input.wheel_width)
        .padding(t.get(&kivact_core::style::space::SM_VAR))
        .gap(t.get(&kivact_core::style::space::XS_VAR))
        .background(kivact_core::style::color::SURFACE)
        .color(kivact_core::style::color::TEXT)
}
