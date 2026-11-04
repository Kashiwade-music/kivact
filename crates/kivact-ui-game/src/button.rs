//! Role: Toolbar actions, action groups and chevron compositions.
use crate::Tokens;
use kivact::draw::Filter;
use kivact::state::IntoText;
use kivact::style::{Align, ColorToken, Justify, pct, style, use_theme};
use kivact::ui::{Button, ButtonVariant, button};
use kivact::view::{IntoView, Role};
use kivact::view::{NodeBuilder, row, text};
/// Square toolbar action, composed from the standard button.
pub fn tool_button(label: impl IntoText) -> Button {
    button(label).variant(ButtonVariant::Ghost).style(
        style()
            .size(32, 32)
            .padding(0)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .color(ColorToken::TextMuted)
            .hover(
                style()
                    .background(ColorToken::SurfaceAlt)
                    .border(1, ColorToken::Border)
                    .color(ColorToken::Text),
            )
            .selected(
                style()
                    .background(ColorToken::Selection)
                    .color(ColorToken::Accent)
                    .border(1, ColorToken::Accent),
            ),
    )
}
/// Closely spaced group of action buttons.
#[track_caller]
pub fn button_group(children: impl IntoView) -> impl kivact::view::Component {
    row(children)
        .with_node(|root| root.role(Role::Group))
        .style(style().gap(2).align_items(Align::Center))
}

/// Dense bordered toolbar.
#[track_caller]
pub fn toolbar(children: impl IntoView) -> impl kivact::view::Component {
    row(children)
        .with_node(|root| root.role(Role::Toolbar))
        .style(
            style()
                .align_items(Align::Center)
                .gap(2)
                .padding(3)
                .background(ColorToken::Surface)
                .border(1, ColorToken::Border),
        )
}

/// Large chevron action with a small machine caption.
#[track_caller]
pub fn chevron(label: impl IntoText, caption: impl IntoText) -> Button {
    let label = label.into_text_value();
    let content = row((
        text(label).style(move || {
            let t = &use_theme().get();
            style()
                .font_family(t.ext::<Tokens>().display_family.as_str())
                .font_size(20)
                .letter_spacing(1.6)
                .weight(800)
                .italic(true)
                .color(ColorToken::AccentText)
        }),
        crate::mono(caption).style(style().color(ColorToken::AccentText)),
    ));
    button(label)
        .variant(ButtonVariant::Primary)
        .hide_label(true)
        .style(move || {
            let t = &use_theme().get();
            style()
                .shadow(kivact::draw::Shadow::new(
                    0,
                    0,
                    0,
                    kivact::style::Color::TRANSPARENT,
                ))
                .height(48)
                .align_self(Align::Start)
                .padding_left(22)
                .padding_right(30)
                .color(t.colors().accent_text)
                .background(
                    kivact::draw::linear_gradient(100)
                        .colors([t.colors().accent, t.ext::<Tokens>().cyan]),
                )
                .clip_path(
                    kivact::draw::Shape::polygon()
                        .point(0, 0)
                        .point(pct(100) - 16, 0)
                        .point(pct(100), pct(50))
                        .point(pct(100) - 16, pct(100))
                        .point(0, pct(100)),
                )
                .hover(style().filter(Filter::new().brightness(1.08)))
                .pressed(style().translate(2, 0))
        })
        .child(content.style(style().gap(12).align_items(Align::Center)))
}
