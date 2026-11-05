//! Role: Compose editable controls with fixed prefix and suffix content.
use kivact::state::IntoText;
use kivact::style::use_theme;
use kivact::style::{Align, ColorToken, Position, pct, style};
use kivact::view::{NodeBuilder, node, row};

use crate::Tokens;

/// Icons or captions placed inside a field without replacing native editing.
#[track_caller]
pub fn input_adorned(
    prefix: impl kivact::view::IntoView,
    control: kivact::ui::TextInput,
    suffix: impl kivact::view::IntoView,
) -> impl kivact::view::Component {
    row((
        control.style(style().width(pct(100)).padding_left(34).padding_right(52)),
        node().child(prefix).style(
            style()
                .position(Position::Absolute)
                .left(10)
                .align_self(Align::Center)
                .color(ColorToken::TextMuted),
        ),
        node().child(suffix).style(
            style()
                .position(Position::Absolute)
                .right(8)
                .align_self(Align::Center)
                .color(ColorToken::TextMuted),
        ),
    ))
    .style(style().width(pct(100)).align_items(Align::Center))
}

/// Fixed prefix and suffix around an editable field.
#[track_caller]
pub fn input_group(
    prefix: impl IntoText,
    control: kivact::ui::TextInput,
    suffix: impl IntoText,
) -> impl kivact::view::Component {
    input_group_content(crate::mono(prefix), control, crate::mono(suffix))
}

/// Fixed prefix and suffix slots accepting icons or arbitrary public views.
#[track_caller]
pub fn input_group_content(
    prefix: impl kivact::view::IntoView,
    control: kivact::ui::TextInput,
    suffix: impl kivact::view::IntoView,
) -> impl kivact::view::Component {
    let addon = |content: kivact::view::View| {
        node()
            .style(move || {
                let t = &use_theme().get();
                style()
                    .height(t.ext::<Tokens>().control_height)
                    .padding_x(10)
                    .align_items(Align::Center)
                    .background(t.ext::<Tokens>().surface_3)
                    .border(1, t.ext::<Tokens>().line_strong)
            })
            .child(content)
    };
    row((
        addon(prefix.into_view()),
        control.style(style().grow(1).min_width(0)),
        addon(suffix.into_view()),
    ))
    .style(style().width(pct(100)).align_items(Align::Center))
}
