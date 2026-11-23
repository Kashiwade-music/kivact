//! Role: The console's table cell looks for the standard data table's columns.

use crate::surface::mono;
use crate::variants::AvatarVariant;
use kivact::component;
use kivact::i18n::message;
use kivact::interaction::Tone;
use kivact::state::IntoValue;
use kivact::style::Color;
use kivact::style::color::{ACCENT, TEXT_MUTED};
use kivact::style::{Align, pct, style};
use kivact::ui::ControlSize;
use kivact::view::{Element, column, row, text};

/// An identifier in the mono type, in the accent. Use it in a column's cell.
#[component]
pub fn identifier_cell(value: String) -> impl Element {
    mono(value.into_value()).style(style().font_size(11.5).color(ACCENT))
}

/// A title in bold over a muted description. Use it in a column's cell.
#[component]
pub fn detail_cell(title: String, description: String) -> impl Element {
    column((
        text(title).style(style().font_size(12.5).font_weight(700)),
        text(description).style(style().font_size(10.5).color(TEXT_MUTED)),
    ))
    .style(style().gap(1))
}

/// An avatar of `color` beside the user's name. Use it in a column's cell.
#[component]
pub fn user_cell(name: String, initials: String, color: Color) -> impl Element {
    row((
        kivact::ui::avatar(initials.clone().into_value())
            .fallback_content(move || text(initials.clone()))
            .variant(AvatarVariant { side: 24., color }),
        text(name).style(style().font_size(12.5)),
    ))
    .style(style().gap(10).align_items(Align::Center))
}

/// A badge of `label` in `tone`. Use it in a column's cell.
#[component]
pub fn badge_cell(label: String, tone: Tone) -> impl Element {
    kivact::ui::badge(label.into_value()).tone(tone)
}

/// A thin progress bar in `tone` with its percentage, from 0 to 100.
/// Use it in a column's cell.
#[component]
pub fn progress_cell(value: f64, tone: Tone) -> impl Element {
    row((
        kivact::ui::progress((Some(value / 100.0)).into_value())
            .name(message("game.table.progress", "Progress"))
            .variant(tone)
            .size(ControlSize::Small)
            .style(style().flex_grow(1.)),
        mono(format!("{value:.0}%").into_value()),
    ))
    .style(style().gap(8).align_items(Align::Center).width(pct(100)))
}
