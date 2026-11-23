//! Role: Style a keyboard-reachable text trigger for standard hover previews.
use crate::color as c;
use kivact::component;
use kivact::state::Value;
use kivact::style::{TextDecorationLine, style};
use kivact::view::text;

/// A keyboard-reachable text trigger with the native solid underline fallback.
#[component]
pub fn hover_card_text(label: Value<String>) -> impl Element {
    text(label).focusable(true).style(|| {
        style()
            .text_decoration(TextDecorationLine::UNDERLINE)
            .text_decoration_color(c::BORDER_EMPHASIZED)
            .text_underline_offset(2)
    })
}
