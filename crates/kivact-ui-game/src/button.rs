//! Role: Toolbar actions, the square tool button and the chevron action.
use crate::color::CYAN;
use crate::fonts::DISPLAY_FAMILY_VAR;
use crate::surface::mono;
use kivact::component;
use kivact::state::Value;
use kivact::style::color::{ACCENT, ACCENT_TEXT, BORDER, SELECTION, SURFACE_ALT, TEXT, TEXT_MUTED};
use kivact::style::{
    Align, Color, Filter, FontStyle, Justify, Shadow, Shape, linear_gradient, pct, style, use_theme,
};
use kivact::ui::{Button, ButtonVariant, button};
use kivact::view::{Element, row, text};

/// The square tool button: a ghost button of 32 px that tints on hover and
/// on selection.
#[component]
pub fn tool_button(label: Value<String>) -> impl Element {
    tool_button_of(label)
}

/// The tool button as a builder, for the kit's own menus to add an icon to.
pub(crate) fn tool_button_of(label: Value<String>) -> Button {
    button(label).variant(ButtonVariant::Ghost).style(
        style()
            .size(32, 32)
            .padding(0)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .color(TEXT_MUTED)
            .hover(
                style()
                    .background(SURFACE_ALT)
                    .border(1, BORDER)
                    .color(TEXT),
            )
            .selected(
                style()
                    .background(SELECTION)
                    .color(ACCENT)
                    .border(1, ACCENT),
            ),
    )
}

/// The large chevron action: the display type over a small mono caption,
/// on a slanted accent-to-cyan gradient.
#[component]
pub fn chevron(label: Value<String>, caption: Value<String>) -> impl Element {
    let content = row((
        text(label).style(move || {
            let t = use_theme().get();
            style()
                .font_family(t.get(&DISPLAY_FAMILY_VAR).as_str().unwrap_or_default())
                .font_size(20)
                .letter_spacing(1.6)
                .font_weight(800)
                .font_style(FontStyle::Italic)
                .color(ACCENT_TEXT)
        }),
        mono(caption).style(style().color(ACCENT_TEXT)),
    ));
    button(label)
        .variant(ButtonVariant::Primary)
        .hide_label(true)
        .style(move || {
            style()
                .shadow(Shadow::new(0, 0, 0, Color::TRANSPARENT))
                .height(48)
                .align_self(Align::Start)
                .padding_left(22)
                .padding_right(30)
                .color(ACCENT_TEXT)
                .background(linear_gradient(100).colors([ACCENT, CYAN]))
                .clip_path(
                    Shape::polygon()
                        .point(0, 0)
                        .point(pct(100) - 16, 0)
                        .point(pct(100), pct(50))
                        .point(pct(100) - 16, pct(100))
                        .point(0, pct(100)),
                )
                .hover(style().filter(Filter::new().brightness(1.08)))
                .active(style().translate(2, 0))
        })
        .children(vec![
            content
                .style(style().gap(12).align_items(Align::Center))
                .into_view(),
        ])
}
