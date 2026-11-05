//! Role: Basic recipes for the canonical follow-end chat thread and its real parts.
use crate::Tokens;
use kivact::style::{Color, Direction, Position, Style, Theme, pct, style};
use kivact::ui::{ChatThreadRecipes, ChatThreadState, Look};
pub(crate) fn recipes(r: ChatThreadRecipes) -> ChatThreadRecipes {
    r.root(|_| root)
        .viewport(|_| viewport)
        .content(|_| content)
        .scroll_button(|_| jump)
}
fn root(_: &Theme, _: &Look<ChatThreadState>) -> Style {
    style()
        .position(Position::Relative)
        .direction(Direction::Column)
        .min_height(0)
        .min_width(0)
}
fn viewport(t: &Theme, _: &Look<ChatThreadState>) -> Style {
    style()
        .grow(1)
        .shrink(1)
        .min_height(0)
        .min_width(0)
        .width(pct(100))
        .scrollbar_width(10)
        .scrollbar_color(t.ext::<Tokens>().border, Color::TRANSPARENT)
}
fn content(_: &Theme, _: &Look<ChatThreadState>) -> Style {
    style()
        .direction(Direction::Column)
        .gap(24)
        .shrink(0)
        .min_height(pct(100))
        .min_width(0)
}
fn jump(_: &Theme, _: &Look<ChatThreadState>) -> Style {
    style()
        .size(28, 28)
        .padding(0)
        .position(Position::Absolute)
        .left(pct(50) - 14)
        .bottom(16)
}
