//! Role: Style a keyboard-reachable text trigger for standard hover previews.
/// A keyboard-reachable text trigger with the native solid underline fallback.
pub fn hover_card_text(label: impl kivact::state::IntoText) -> kivact::view::Node {
    kivact::view::text(label).focusable(true).style(|| {
        kivact::style::style()
            .text_decoration(kivact::style::TextDecorationLine::UNDERLINE)
            .text_decoration_color(
                kivact::style::use_tokens::<crate::Tokens>()
                    .colors
                    .border_emphasized,
            )
            .text_underline_offset(2)
    })
}
