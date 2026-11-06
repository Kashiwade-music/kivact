//! Role: Lyra recipes for canonical static table sections and cells.
use crate::Tokens;
use kivact::custom::PartState;
use kivact::style::{Align, Direction, Overflow, Style, Theme, pct, style};
use kivact::ui::{Look, TableRecipes};
pub(crate) fn recipes(r: TableRecipes) -> TableRecipes {
    r.root(|_| root)
        .header(|_| header)
        .body(|_| body)
        .footer(|_| footer)
        .row(|_| row)
        .head(|_| head)
        .cell(|_| cell)
}
fn root(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .direction(Direction::Column)
        .width(pct(100))
        .font_size(12)
        .overflow(Overflow::Hidden)
}
fn header(_: &Theme, _: &Look<PartState>) -> Style {
    style().direction(Direction::Column)
}
fn body(_: &Theme, _: &Look<PartState>) -> Style {
    style().direction(Direction::Column).margin_bottom(-1)
}
fn footer(t: &Theme, _: &Look<PartState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .direction(Direction::Column)
        .margin_bottom(-1)
        .border_top(1, p.border)
        .background(p.muted.with_alpha(0.5))
        .weight(500)
}
fn row(t: &Theme, _: &Look<PartState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .width(pct(100))
        .border_bottom(1, p.border)
        .hover(style().background(p.muted.with_alpha(0.5)))
        .selected(style().background(p.muted))
}
fn head(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .padding_x(8)
        .min_width(0)
        .text_wrap(false)
        .color(t.ext::<Tokens>().foreground)
        .height(40)
        .weight(500)
}
fn cell(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .padding(8)
        .min_width(0)
        .text_wrap(false)
        .color(t.ext::<Tokens>().foreground)
}
