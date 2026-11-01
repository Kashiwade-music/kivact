//! Role: Game recipes for the standard list and data table.
//!
//! The console's table: mono type, 34 px headers on the alternate surface,
//! 160 px columns that share the width beyond that, a cyan-tinted row on
//! hover and an accent edge on a selected row. Drawn by `kivact::ui::list`
//! and `kivact::ui::data_table`.

use crate::tokens::Tokens;
use kivact::style::{ColorToken, Theme, style};
use kivact::ui::{DataTableRecipes, ListRecipes};

pub(crate) fn list(r: ListRecipes) -> ListRecipes {
    r.item(|prev| {
        move |t: &Theme, s: &_| {
            let p = t.get::<Tokens>();
            prev(t, s)
                .min_height(p.row_height)
                .padding_x(12)
                .radius(0)
                .hover(style().background(p.primary_weak))
                .selected(
                    style()
                        .background(p.primary_weak)
                        .border_left(3, ColorToken::Accent),
                )
        }
    })
}

pub(crate) fn data_table(r: DataTableRecipes) -> DataTableRecipes {
    r.frame(|prev| move |t: &Theme, s: &_| prev(t, s).radius(0).background(ColorToken::Surface))
        .header(|prev| {
            move |t: &Theme, s: &_| {
                let p = t.get::<Tokens>();
                prev(t, s)
                    .width(160)
                    .min_width(160)
                    .min_height(34)
                    .padding_x(12)
                    .font_family(p.mono_family.as_str())
                    .font_size(11.5)
                    .background(ColorToken::SurfaceAlt)
            }
        })
        .row(|prev| {
            move |t: &Theme, s: &_| {
                let p = t.get::<Tokens>();
                prev(t, s)
                    .group()
                    .min_height(s.height.unwrap_or(p.row_height))
                    .hover(style().background(p.primary_weak))
                    .selected(
                        style()
                            .background(p.primary_weak)
                            .border_left(3, ColorToken::Accent),
                    )
            }
        })
        .cell(|prev| {
            move |t: &Theme, s: &_| {
                let p = t.get::<Tokens>();
                prev(t, s)
                    .width(160)
                    .min_width(160)
                    .padding_x(12)
                    .font_family(p.mono_family.as_str())
                    .font_size(11.5)
            }
        })
}
