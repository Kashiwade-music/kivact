//! Role: Present a quotation and optional attribution with a logical start rule.
use crate::color as c;
use crate::space as s;
use crate::text::{TextColor, TextKind, styled_text};
use kivact::component;
use kivact::state::Value;
use kivact::style::FlexDirection;
use kivact::style::{style, use_theme};
use kivact::view::Element;
use kivact::view::ReadingDirection;
use kivact::view::View;
use kivact::view::node;
use kivact::view::use_direction;

/// Render a quotation with caller-supplied content and optional attribution.
#[component]
pub fn blockquote(content: View, cite: Option<Value<String>>) -> impl kivact::view::Element {
    node()
        .with_node(move |node| {
            node.base_style(|| {
                let rule = use_theme().get().get(&s::SPACE_S0_5_VAR);
                let look = style()
                    .flex_direction(FlexDirection::Column)
                    .min_width(0)
                    .color(c::TEXT_SECONDARY);
                if use_direction() == ReadingDirection::Rtl {
                    look.border_right(rule, c::BORDER_EMPHASIZED)
                        .padding_right(s::SPACE_S4)
                } else {
                    look.border_left(rule, c::BORDER_EMPHASIZED)
                        .padding_left(s::SPACE_S4)
                }
            })
        })
        .child(content)
        .child(cite.map(|value| {
            node().style(|| style().margin_top(s::SPACE_S2)).child(
                styled_text(value)
                    .kind(TextKind::Supporting)
                    .color(TextColor::Inherit),
            )
        }))
}

/// Render a plain-text quotation.
#[component]
pub fn blockquote_text(value: Value<String>) -> impl kivact::view::Element {
    blockquote(styled_text(value).kind(TextKind::Inherit))
}
