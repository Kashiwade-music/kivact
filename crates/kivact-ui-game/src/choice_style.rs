//! Role: Styles of the radio group and tabs looks: segments, cards, navigation, boxed.

use kivact::draw::Path;
use kivact::geometry::Point;
use kivact::style::{Align, Color, ColorToken, Direction, Justify, Position, Style, pct, style};

/// A segment or a card takes no indicator: out of the row, with no size.
pub(crate) fn hidden_indicator() -> Style {
    style().position(Position::Absolute).size(0, 0).clip(true)
}

pub(crate) fn segment_group() -> Style {
    style()
        .direction(Direction::Row)
        .gap(0)
        .align_items(Align::Center)
}

/// What a chosen segment adds to its button look: the accent fill.
pub(crate) fn segment_chosen() -> Style {
    style()
        .background(ColorToken::Accent)
        .color(ColorToken::AccentText)
}

pub(crate) fn card_group() -> Style {
    style()
        .direction(Direction::Row)
        .gap(8)
        .width(pct(100))
        .justify_content(Justify::SpaceBetween)
}

/// A card in a column; a chosen one has a corner mark in the accent.
pub(crate) fn card_item(chosen: bool) -> Style {
    let s = style()
        .direction(Direction::Column)
        .grow(1)
        .padding_x(12)
        .padding_y(10)
        .gap(4)
        .background(ColorToken::SurfaceAlt)
        .border(1, ColorToken::Border)
        .selected(
            style()
                .background(ColorToken::Selection)
                .border_color(ColorToken::Accent),
        )
        .hover(style().border_color(ColorToken::Accent));
    if chosen {
        // The mark is drawn by the item's style, which is read again when
        // the choice changes; a state style cannot draw.
        s.paint(|d, size| {
            d.fill_path(
                &Path::new()
                    .move_to(Point::new(size.width - 18., 0.))
                    .line_to(Point::new(size.width, 0.))
                    .line_to(Point::new(size.width, 18.))
                    .close(),
                ColorToken::Accent,
            );
        })
    } else {
        s
    }
}

pub(crate) fn navigation_list() -> Style {
    style().width(pct(100)).border(1, ColorToken::Border).gap(0)
}

pub(crate) fn navigation_item() -> Style {
    style()
        .direction(Direction::Row)
        .min_height(36)
        .padding_x(12)
        .gap(8)
        .align_items(Align::Center)
        .border_left(3, Color::TRANSPARENT)
        .border_bottom(1, ColorToken::Border)
        .selected(
            style()
                .background(ColorToken::Selection)
                .color(ColorToken::Accent)
                .border_color(ColorToken::Accent)
                .weight(700),
        )
        .hover(style().background(ColorToken::SurfaceAlt))
}

pub(crate) fn boxed_list() -> Style {
    style()
        .direction(Direction::Row)
        .width(pct(100))
        .padding(3)
        .gap(0)
        .background(ColorToken::SurfaceAlt)
        .border(1, ColorToken::Border)
}

pub(crate) fn boxed_tab() -> Style {
    style()
        .direction(Direction::Row)
        .gap(7)
        .height(30)
        .padding_x(16)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .font_size(12.5)
        .border_left(3, Color::TRANSPARENT)
        .background(ColorToken::SurfaceAlt)
        .color(ColorToken::TextMuted)
        .hover(style().background(ColorToken::Selection))
        .selected(
            style()
                .border_color(ColorToken::Accent)
                .background(ColorToken::Surface)
                .color(ColorToken::Text)
                .weight(700),
        )
}
