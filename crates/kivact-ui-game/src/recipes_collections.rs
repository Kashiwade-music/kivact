//! Role: Game recipes for the standard list and data table.
//!
//! The console's table: mono type, 34 px headers on the alternate surface,
//! 160 px columns that share the width beyond that, a cyan-tinted row on
//! hover and an accent edge on a selected row. Drawn by `kivact::ui::list`
//! and `kivact::ui::data_table`.

use crate::tokens::Tokens;
use kivact::style::{ColorToken, Theme, style};
use kivact::ui::{DataTableRecipes, ItemListRecipes, ListRecipes, TableRecipes, TreeRecipes};

pub(crate) fn list(r: ListRecipes) -> ListRecipes {
    r.item(|prev| {
        move |t: &Theme, s: &_| {
            let p = t.ext::<Tokens>();
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
                let p = t.ext::<Tokens>();
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
                let p = t.ext::<Tokens>();
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
                let p = t.ext::<Tokens>();
                prev(t, s)
                    .width(160)
                    .min_width(160)
                    .padding_x(12)
                    .font_family(p.mono_family.as_str())
                    .font_size(11.5)
            }
        })
}

pub(crate) fn tree(r: TreeRecipes) -> TreeRecipes {
    r.root(|base| move |t, s| base(t, s).padding_y(4))
        .item(|base| {
            move |t, s| {
                base(t, s)
                    .height(26)
                    .min_height(26)
                    .hover(style().background(ColorToken::SurfaceAlt))
                    .selected(
                        style()
                            .background(ColorToken::Accent)
                            .color(ColorToken::AccentText),
                    )
            }
        })
        .marker(|base| move |t, s| base(t, s).font_size(11).color(ColorToken::TextMuted))
}
pub(crate) fn table(r: TableRecipes) -> TableRecipes {
    r.root(|base| move |t, s| base(t, s).font_family(t.ext::<Tokens>().mono_family.as_str()))
        .header(|base| move |t, s| base(t, s).background(ColorToken::SurfaceAlt))
        .row(|base| {
            move |t, s| {
                base(t, s)
                    .min_height(t.ext::<Tokens>().row_height)
                    .hover(style().background(t.ext::<Tokens>().primary_weak))
            }
        })
        .head(|base| move |t, s| base(t, s).padding_x(12).font_size(11.5))
        .cell(|base| move |t, s| base(t, s).padding_x(12).font_size(11.5))
}
pub(crate) fn item_list(r: ItemListRecipes) -> ItemListRecipes {
    r.item(|base| {
        move |t, s| {
            base(t, s)
                .min_height(t.ext::<Tokens>().row_height)
                .border_bottom(1, t.ext::<Tokens>().line_strong)
                .hover(style().background(t.ext::<Tokens>().primary_weak))
        }
    })
    .title(|base| move |t, s| base(t, s).font_size(12.5).weight(700))
    .description(|base| move |t, s| base(t, s).font_size(11.5))
}
