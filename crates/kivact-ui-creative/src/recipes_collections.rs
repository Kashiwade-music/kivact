//! Role: Creative recipes for the standard list and data table.
//!
//! The dense table of the editors: 30 px headers on the header surface,
//! rows of the kit's row height with an edge in the row's own color,
//! 9 px cell padding and small type. Drawn by `kivact::ui::list` and
//! `kivact::ui::data_table`.

use crate::tokens::Tokens;
use kivact::style::{ColorToken, Theme, style};
use kivact::ui::{DataTableRecipes, ListRecipes};

pub(crate) fn list(r: ListRecipes) -> ListRecipes {
    r.item(|prev| {
        move |t: &Theme, s: &_| {
            let p = t.ext::<Tokens>();
            prev(t, s)
                .min_height(p.row)
                .padding_x(9)
                .font_size(11.5)
                .color(p.text)
                .selected(style().background(p.selection))
        }
    })
}

pub(crate) fn data_table(r: DataTableRecipes) -> DataTableRecipes {
    r.frame(|prev| {
        move |t: &Theme, s: &_| {
            let p = t.ext::<Tokens>();
            prev(t, s).radius(0).border_color(p.hair)
        }
    })
    .header(|prev| {
        move |t: &Theme, s: &_| {
            let p = t.ext::<Tokens>();
            prev(t, s)
                .min_height(30)
                .padding_x(9)
                .font_size(10.5)
                .color(p.dim)
                .background(p.header)
                .border_color(p.hair)
        }
    })
    .row(|prev| {
        move |t: &Theme, s: &_| {
            let p = t.ext::<Tokens>();
            prev(t, s)
                .min_height(s.height.unwrap_or(p.row + 4.0))
                .color(p.text)
                .border_color(p.hair)
                .border_left(3, s.color.unwrap_or(p.hair))
                .hover(style().background(ColorToken::Hover))
                .selected(style().background(p.selection))
        }
    })
    .cell(|prev| move |t: &Theme, s: &_| prev(t, s).padding_x(9).font_size(11.5))
}

/// Static tables share the editor table's dense header and cell typography.
pub(crate) fn table(r: kivact::ui::TableRecipes) -> kivact::ui::TableRecipes {
    r.header(|prev| move |t: &Theme, s: &_| prev(t, s).background(t.ext::<Tokens>().header))
        .head(|prev| {
            move |t: &Theme, s: &_| prev(t, s).font_size(10.5).color(t.ext::<Tokens>().dim)
        })
        .cell(|prev| move |t: &Theme, s: &_| prev(t, s).padding_x(9).font_size(11.5))
        .row(|prev| {
            move |t: &Theme, s: &_| {
                let p = t.ext::<Tokens>();
                prev(t, s)
                    .min_height(p.row)
                    .hover(style().background(ColorToken::Hover))
                    .selected(style().background(p.selection))
            }
        })
}

/// Static rows retain shared semantics while matching the editor's density.
pub(crate) fn item_list(r: kivact::ui::ItemListRecipes) -> kivact::ui::ItemListRecipes {
    r.item(|prev| {
        move |t: &Theme, s: &_| {
            let p = t.ext::<Tokens>();
            prev(t, s)
                .min_height(p.row)
                .padding_x(9)
                .radius(5)
                .hover(style().background(ColorToken::Hover))
                .selected(style().background(p.selection))
        }
    })
    .title(|prev| move |t: &Theme, s: &_| prev(t, s).font_size(11.5).color(t.ext::<Tokens>().text))
    .description(|prev| {
        move |t: &Theme, s: &_| prev(t, s).font_size(10.5).color(t.ext::<Tokens>().muted)
    })
}
