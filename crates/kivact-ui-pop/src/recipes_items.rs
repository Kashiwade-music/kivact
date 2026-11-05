//! Role: Style canonical static collections and full-hue item media in Pop.
use crate::{Hue, Tokens};
use kivact::state::{IntoValue, Value};
use kivact::style::{Align, Direction, Justify, Theme, style};
use kivact::ui::{ItemListRecipes, TableRecipes, TreeRecipes, Variant};

/// Reactive hue of an item row's leading media tile.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemHue(pub Value<Hue>);
impl ItemHue {
    pub fn new(hue: impl IntoValue<Hue>) -> Self {
        Self(hue.into_value())
    }
}
impl Variant for ItemHue {}

pub(crate) fn item_list(r: ItemListRecipes) -> ItemListRecipes {
    r.root(|prev| move |t: &Theme, s: &_| prev(t, s).gap(0))
        .item(|prev| {
            move |t: &Theme, s: &_| {
                prev(t, s)
                    .padding_x(8)
                    .padding_y(10)
                    .gap(0)
                    .radius(8)
                    .hover(style().background(t.ext::<Tokens>().surface_active))
            }
        })
        .body(|prev| move |t: &Theme, s: &_| prev(t, s).gap(12).align_items(Align::Center))
        .description(|prev| {
            move |t: &Theme, s: &_| prev(t, s).font_size(12).color(t.ext::<Tokens>().text_muted)
        })
        .media(|_| {
            |t: &Theme, s: &kivact::ui::Look<kivact::ui::ItemRowState>| {
                let p = t.ext::<Tokens>();
                let hue = s.variant::<ItemHue>().map_or(Hue::Gray, |h| h.0.get());
                let (bg, fg) = p.hue(hue);
                style()
                    .size(36, 36)
                    .radius(8)
                    .background(bg)
                    .color(fg)
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .shrink(0)
            }
        })
}
pub(crate) fn table(r: TableRecipes) -> TableRecipes {
    r.root(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .radius(14)
                .border(1, t.ext::<Tokens>().line)
                .background(t.ext::<Tokens>().surface)
        }
    })
    .header(|prev| move |t: &Theme, s: &_| prev(t, s).background(t.ext::<Tokens>().surface_muted))
    .head(|prev| {
        move |t: &Theme, s: &_| prev(t, s).font_size(12).color(t.ext::<Tokens>().text_muted)
    })
    .cell(|prev| move |t: &Theme, s: &_| prev(t, s).font_size(13).padding_x(12).padding_y(10))
}
pub(crate) fn tree(r: TreeRecipes) -> TreeRecipes {
    r.item(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .radius(8)
                .font_size(13)
                .hover(style().background(t.ext::<Tokens>().surface_muted))
        }
    })
}
