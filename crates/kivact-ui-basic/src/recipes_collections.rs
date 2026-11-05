//! Role: Lyra recipes for the standard list and data table.
//!
//! The looks of shadcn's data-table recipe: a bordered, rounded grid with
//! 40 px headers, muted rows on hover and selection, and the table's cells
//! at 8 px padding. The standard components draw them with
//! `kivact::ui::list` and `kivact::ui::data_table`.

use crate::ColorName;
use crate::tokens::Tokens;
use kivact::style::{Theme, style};
use kivact::ui::{DataTableRecipes, ListRecipes};

pub(crate) fn list(r: ListRecipes) -> ListRecipes {
    r.item(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .padding_x(8)
                .min_height(32)
                .radius(6)
                .hover(style().background(ColorName::Accent))
                .selected(style().background(ColorName::Accent))
        }
    })
    .description(|_| |_: &Theme, _: &_| style().font_size(12).color(ColorName::MutedForeground))
}

pub(crate) fn data_table(r: DataTableRecipes) -> DataTableRecipes {
    r.frame(|prev| move |t: &Theme, s: &_| prev(t, s).border_color(ColorName::Border).radius(6))
        .toolbar(|prev| move |t: &Theme, s: &_| prev(t, s).padding_x(0).padding_y(16))
        .footer(|prev| move |t: &Theme, s: &_| prev(t, s).padding_x(0).padding_y(16))
        .header(|prev| {
            move |t: &Theme, s: &_| {
                prev(t, s)
                    .min_height(40)
                    .padding_x(8)
                    .weight(500)
                    .color(ColorName::Foreground)
                    .border_color(ColorName::Border)
            }
        })
        .row(|prev| {
            move |t: &Theme, s: &_| {
                prev(t, s)
                    .border_color(ColorName::Border)
                    .hover(style().background(t.ext::<Tokens>().muted.with_alpha(0.5)))
                    .selected(style().background(ColorName::Muted))
            }
        })
        .cell(|prev| move |t: &Theme, s: &_| prev(t, s).padding(8).color(ColorName::Foreground))
}
