//! Role: The segmented radio look, the check row style and choice list appearance.

use crate::Tokens;
use kivact::draw::Shadow;
use kivact::style::{Align, Color, ColorToken, Direction, Justify, Style, Theme, ThemeMode, style};

/// The row of a check: the drawn part beside the label. A disabled row
/// takes the theme's `disabled_opacity`, as every focus stop does.
pub(crate) fn check_style(_: &Theme) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(12)
        .min_height(22)
        .color(ColorToken::Text)
}

const SELECT_OPTION_HEIGHT: f64 = 32.0;

pub(crate) fn select_option(t: &Theme) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .height(SELECT_OPTION_HEIGHT)
        .padding_x(8)
        .radius(4)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .hover(style().background(p.surface_active))
        .selected(style().background(p.surface_active))
}

pub(crate) fn select_list(t: &Theme) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .min_width(220)
        .padding(4)
        .background(p.surface)
        .border(1, p.line)
        .radius(8)
        .shadow(Shadow::new(0, 4, 6, ColorToken::Shadow))
}

pub(crate) fn choice_option(t: &Theme, pill: bool) -> Style {
    let p = t.ext::<Tokens>();
    let s = style()
        .height(if pill { 30 } else { 36 })
        .padding_x(12)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(6)
        .font_size(14)
        .weight(500)
        .color(p.text_muted)
        .hover(style().color(p.text));
    if pill {
        s.radius(7).selected(
            style()
                .background(if p.mode == ThemeMode::Dark {
                    crate::tokens::selected_track()
                } else {
                    p.surface
                })
                .color(p.text)
                .shadow(Shadow::new(0, 1, 3, ColorToken::Shadow)),
        )
    } else {
        s.border_bottom(2, Color::TRANSPARENT)
            .selected(style().border_bottom(2, p.blue).color(p.text))
    }
}

pub(crate) fn choice_list(t: &Theme, pill: bool) -> Style {
    let p = t.ext::<Tokens>();
    if pill {
        style().padding(3).radius(10).background(p.surface_active)
    } else {
        style().gap(4).border_bottom(1, p.line)
    }
}
