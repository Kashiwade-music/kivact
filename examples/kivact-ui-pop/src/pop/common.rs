//! Role: Shared layout and text builders for Pop catalog specimens.

use kivact::prelude::{ColorToken, IntoView, Node, Role, column, grid, row, text};
use kivact::style::{Align, style};
use kivact::style::{GridPlacement, Track};
use kivact_ui_pop as ui;

pub(super) fn cards_grid(children: impl IntoView) -> Node {
    grid(children).style(
        style()
            .grid_template_columns(Track::repeat(12, Track::fr(1)))
            .gap(16),
    )
}

pub(super) fn columns(children: impl IntoView, count: u16) -> Node {
    grid(children).style(
        style()
            .grid_template_columns(Track::repeat(count, Track::fr(1)))
            .gap(16),
    )
}

pub(super) fn card(title: &str, content: impl IntoView) -> kivact::ui::Card {
    kivact::ui::card((
        row(text(title.to_string())
            .role(Role::Heading)
            .style(style().font_size(15).weight(600).line_height(1.2)))
        .style(
            style()
                .align_items(Align::Center)
                .min_height(20)
                .gap(8)
                .margin_bottom(16),
        ),
        content,
    ))
}

pub(super) fn half<C: kivact::view::Component>(v: C) -> C {
    v.style(style().grid_column(GridPlacement::span(6)))
}

pub(super) fn third<C: kivact::view::Component>(v: C) -> C {
    v.style(style().grid_column(GridPlacement::span(4)))
}

pub(super) fn wrap(v: impl IntoView) -> Node {
    row(v).style(style().gap(8).wrap(true).align_items(Align::Center))
}

pub(super) fn section(number: &str, title: &str, subtitle: &str, content: impl IntoView) -> Node {
    column((
        ui::section_heading(number, title.to_string(), subtitle.to_string()),
        content,
    ))
}

pub(super) fn muted(s: &str) -> kivact::view::Node {
    text(s.to_string()).style(style().color(ColorToken::TextMuted))
}

/// The meaning a notification of a hue has.
pub(super) fn tone_of(hue: ui::Hue) -> kivact::interaction::Tone {
    use kivact::interaction::Tone;
    match hue {
        ui::Hue::Mint => Tone::Success,
        ui::Hue::Yellow => Tone::Warning,
        ui::Hue::Red => Tone::Danger,
        ui::Hue::Blue | ui::Hue::Pink => Tone::Info,
        ui::Hue::Gray => Tone::Neutral,
    }
}
