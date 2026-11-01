//! Role: Halo recipes for the standard list and data table.
//!
//! A rounded surface with 40 px headers, 52 px rows and 160 px columns that
//! scroll sideways, muted on hover and selection, drawn by
//! `kivact::ui::list` and `kivact::ui::data_table`.

use crate::Color;
use kivact::style::{Theme, style};
use kivact::ui::{DataTableRecipes, ListRecipes};

pub(crate) fn list(r: ListRecipes) -> ListRecipes {
    r.item(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .padding_x(12)
                .min_height(40)
                .radius(10)
                .weight(500)
                .hover(style().background(Color::SurfaceMuted))
                .selected(style().background(Color::SurfaceActive))
        }
    })
    .description(|_| |_: &Theme, _: &_| style().font_size(13).color(Color::TextMuted))
}

pub(crate) fn data_table(r: DataTableRecipes) -> DataTableRecipes {
    r.frame(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .background(Color::Surface)
                .border_color(Color::Line)
                .radius(14)
        }
    })
    .toolbar(|prev| {
        move |t: &Theme, s: &_| prev(t, s).padding(16).gap(8).border_bottom(1, Color::Line)
    })
    .footer(|prev| move |t: &Theme, s: &_| prev(t, s).padding_x(16).padding_y(12).gap(4))
    .header(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .width(160)
                .min_width(160)
                .min_height(40)
                .padding_x(12)
                .font_size(13)
                .weight(500)
                .color(Color::TextMuted)
                .border_color(Color::Line)
        }
    })
    .row(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .min_height(s.height.unwrap_or(52.0))
                .border_color(Color::Line)
                .hover(style().background(Color::SurfaceMuted))
                .selected(style().background(Color::SurfaceMuted))
        }
    })
    .cell(|prev| move |t: &Theme, s: &_| prev(t, s).width(160).min_width(160).padding_x(12))
}
