//! Role: Draw static Game lists, table cells and hierarchy rows.
use crate::Tokens;
use kivact::style::{ColorToken, style};
use kivact::ui::{ItemListRecipes, TableRecipes, TreeRecipes};

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
