//! Role: Styles of the radio group and tabs looks: segments, cards, navigation, boxed.
use kivact::draw::Path;
use kivact::geometry::Point;
use kivact::style::color::{ACCENT, ACCENT_TEXT, BORDER, SURFACE, TEXT, TEXT_MUTED};
use kivact::style::{Align, Color, FlexDirection, Justify, Position, Style, pct, style};

/// A segment or a card takes no indicator: out of the row, with no size.
pub(crate) fn hidden_indicator() -> Style {
    style().position(Position::Absolute).size(0, 0).clip(true)
}

pub(crate) fn segment_group() -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .gap(0)
        .align_items(Align::Center)
}

/// What a chosen segment adds to its button look: the accent fill.
pub(crate) fn segment_chosen() -> Style {
    style().background(ACCENT).color(ACCENT_TEXT)
}

pub(crate) fn card_group() -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .gap(8)
        .width(pct(100))
        .justify_content(Justify::SpaceBetween)
}

/// A card in a column; a chosen one has a corner mark in the accent.
pub(crate) fn card_item(chosen: bool) -> Style {
    let s = style()
        .flex_direction(FlexDirection::Column)
        .flex_grow(1.)
        .padding_x(12)
        .padding_y(10)
        .gap(4)
        .background(kivact::style::color::SURFACE_ALT)
        .border(1, BORDER)
        .selected(
            style()
                .background(kivact::style::color::SELECTION)
                .border_color(ACCENT),
        )
        .hover(style().border_color(ACCENT));
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
                ACCENT,
            );
        })
    } else {
        s
    }
}

pub(crate) fn navigation_list() -> Style {
    style().width(pct(100)).border(1, BORDER).gap(0)
}

pub(crate) fn navigation_item() -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .min_height(36)
        .padding_x(12)
        .gap(8)
        .align_items(Align::Center)
        .border_left(3, Color::TRANSPARENT)
        .border_bottom(1, BORDER)
        .selected(
            style()
                .background(kivact::style::color::SELECTION)
                .color(ACCENT)
                .border_color(ACCENT)
                .font_weight(700),
        )
        .hover(style().background(kivact::style::color::SURFACE_ALT))
}

pub(crate) fn boxed_list() -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .width(pct(100))
        .padding(3)
        .gap(0)
        .background(kivact::style::color::SURFACE_ALT)
        .border(1, BORDER)
}

pub(crate) fn boxed_tab() -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .gap(7)
        .height(30)
        .padding_x(16)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .font_size(12.5)
        .border_left(3, Color::TRANSPARENT)
        .background(kivact::style::color::SURFACE_ALT)
        .color(TEXT_MUTED)
        .hover(style().background(kivact::style::color::SELECTION))
        .selected(
            style()
                .border_color(ACCENT)
                .background(SURFACE)
                .color(TEXT)
                .font_weight(700),
        )
}
