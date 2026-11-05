//! Role: Lyra recipes for canonical pagination cells and the actual page binding.
use crate::Tokens;
use kivact::style::{Align, Color, Direction, Justify, Style, Theme, pct, style};
use kivact::ui::{Look, PageItem, PaginationItemState, PaginationRecipes};
use kivact::view::{ReadingDirection, use_direction};
pub(crate) fn recipes(r: PaginationRecipes) -> PaginationRecipes {
    r.root(|_| root).item(|_| item).ellipsis(|_| ellipsis)
}
fn root(_: &Theme, _: &Look<()>) -> Style {
    style()
        .direction(if use_direction().get() == ReadingDirection::Rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        })
        .width(pct(100))
        .justify_content(Justify::Center)
        .align_items(Align::Center)
        .gap(2)
}
fn item(t: &Theme, s: &Look<PaginationItemState>) -> Style {
    let p = t.ext::<Tokens>();
    let dark = p.is_dark();
    let fill = if s.current {
        if dark {
            p.input.with_alpha(0.3)
        } else {
            p.background
        }
    } else {
        Color::TRANSPARENT
    };
    let hover = if dark {
        p.muted.with_alpha(0.5)
    } else {
        p.muted
    };
    let base = style()
        .height(32)
        .min_width(32)
        .radius(0)
        .font_size(12)
        .weight(500)
        .color(p.foreground)
        .background(fill)
        .border(
            1,
            if s.current {
                if dark { p.input } else { p.border }
            } else {
                Color::TRANSPARENT
            },
        )
        .hover(style().background(hover));
    match s.item {
        PageItem::Page(_) => base.padding_x(0),
        PageItem::Previous => base.padding_left(6).padding_right(10),
        PageItem::Next => base.padding_left(10).padding_right(6),
        _ => base.padding_x(10),
    }
}
fn ellipsis(_: &Theme, _: &Look<()>) -> Style {
    style()
        .size(32, 32)
        .shrink(0)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}
