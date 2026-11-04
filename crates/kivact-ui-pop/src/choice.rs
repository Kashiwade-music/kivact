//! Role: The segmented radio look, the check row style and choice list appearance.

use crate::Tokens;
use kivact::draw::{Draw, LineCap, LineJoin, Path, Shadow, Stroke};
use kivact::geometry::{Point, Rect, Size};
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

/// The pill look of a standard radio group, a segmented control:
/// `kivact::ui::radio_group(..).orientation(Orientation::Horizontal)
/// .variant(Segmented)`. Another theme draws the
/// plain radio group.
///
/// A radio group chooses exactly one segment: the arrow keys move and choose
/// (wrapping at the ends), a disabled segment is skipped and the chosen key
/// survives missing rows. Tabs that show content under them are
/// [`kivact::ui::tabs()`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Segmented;

impl kivact::ui::Variant for Segmented {}

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
                    kivact::style::rgb(0x273449)
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

/// A switch's track and thumb in a 32 by 18 box.
pub(crate) fn draw_switch_indicator(d: &mut Draw, s: Size, on: bool, accent: Color, line: Color) {
    d.rounded_rect(s.to_rect(), 9.0, if on { accent } else { line });
    d.rounded_rect(
        Rect::new(if on { 16.0 } else { 2.0 }, 2.0, 14.0, 14.0),
        7.0,
        Color::WHITE,
    );
}

/// A check box in a 16 by 16 box: filled with a check mark when on, with a
/// bar when mixed, an outlined box otherwise.
pub(crate) fn draw_check_indicator(
    d: &mut Draw,
    s: Size,
    on: bool,
    partial: bool,
    accent: Color,
    surface: Color,
    line: Color,
) {
    d.rounded_rect(s.to_rect(), 4.0, if on || partial { accent } else { line });
    if !on && !partial {
        d.rounded_rect(Rect::new(1.0, 1.0, 14.0, 14.0), 3.0, surface);
    }
    if partial {
        d.line(
            Point::new(4.0, 8.0),
            Point::new(12.0, 8.0),
            2.0,
            Color::WHITE,
        );
    } else if on {
        d.stroke_path(
            &Path::new()
                .move_to(Point::new(3.5, 8.0))
                .line_to(Point::new(6.5, 11.0))
                .line_to(Point::new(12.5, 4.5)),
            &Stroke::new(2.0).cap(LineCap::Round).join(LineJoin::Round),
            Color::WHITE,
        );
    }
}
