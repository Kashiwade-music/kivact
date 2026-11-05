//! Role: Lyra recipes for the canonical keyed breadcrumb trail.
use crate::Tokens;
use kivact::style::{Align, Color, Direction, Style, Theme, style};
use kivact::ui::{BreadcrumbRecipes, Look};
use kivact::view::{ReadingDirection, use_direction};
pub(crate) fn recipes(r: BreadcrumbRecipes) -> BreadcrumbRecipes {
    r.root(|_| root)
        .list(|_| list)
        .item(|_| item)
        .separator(|_| separator)
}
fn root(t: &Theme, _: &Look<()>) -> Style {
    style()
        .font_size(12)
        .color(t.ext::<Tokens>().muted_foreground)
}
fn list(_: &Theme, _: &Look<()>) -> Style {
    style()
        .direction(if use_direction().get() == ReadingDirection::Rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        })
        .wrap(true)
        .align_items(Align::Center)
        .gap(6)
}
fn item(t: &Theme, s: &Look<bool>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .direction(Direction::Row)
        .height(kivact::style::Length::Auto)
        .min_width(0)
        .padding(0)
        .gap(0)
        .radius(0)
        .border(0, Color::TRANSPARENT)
        .background(Color::TRANSPARENT)
        .font_size(12)
        .weight(400)
        .text_wrap(false)
        .color(if **s {
            p.foreground
        } else {
            p.muted_foreground
        })
        .hover(style().color(p.foreground).background(Color::TRANSPARENT))
}
fn separator(_: &Theme, _: &Look<()>) -> Style {
    style().shrink(0).align_items(Align::Center)
}
