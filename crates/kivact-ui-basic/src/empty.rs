//! Role: Basic empty-state media variants and recipes.
use crate::Tokens;
use kivact::style::{Align, Direction, Justify, Style, TextAlign, Theme, pct, style};
use kivact::ui::{EmptyStateRecipes, EmptyStateState, Look};
/// The frame around a standard empty-state icon slot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EmptyMediaVariant {
    /// Bare media.
    #[default]
    Default,
    /// A 32 px muted square around an icon.
    Icon,
}
impl kivact::ui::Variant for EmptyMediaVariant {}
pub(crate) fn recipes(r: EmptyStateRecipes) -> EmptyStateRecipes {
    r.root(|_| root)
        .title(|_| title)
        .description(|_| description)
        .icon(|_| icon)
        .actions(|_| actions)
}
fn root(t: &Theme, _: &Look<EmptyStateState>) -> Style {
    style()
        .direction(Direction::Column)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(8)
        .width(pct(100))
        .min_width(0)
        .padding(24)
        .radius(0)
        .color(t.ext::<Tokens>().foreground)
        .font_size(12)
        .text_align(TextAlign::Center)
}
fn title(_: &Theme, _: &Look<EmptyStateState>) -> Style {
    style()
        .font_size(14)
        .line_height(20.0 / 14.0)
        .weight(500)
        .max_width(384)
}
fn description(t: &Theme, _: &Look<EmptyStateState>) -> Style {
    style()
        .line_height(1.625)
        .color(t.ext::<Tokens>().muted_foreground)
        .max_width(384)
}
fn icon(t: &Theme, s: &Look<EmptyStateState>) -> Style {
    let base = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .shrink(0)
        .margin_bottom(8);
    if s.variant::<EmptyMediaVariant>() == Some(&EmptyMediaVariant::Icon) {
        base.size(32, 32)
            .background(t.ext::<Tokens>().muted)
            .color(t.ext::<Tokens>().foreground)
    } else {
        base
    }
}
fn actions(_: &Theme, _: &Look<EmptyStateState>) -> Style {
    style()
        .direction(Direction::Column)
        .align_items(Align::Center)
        .gap(10)
        .width(pct(100))
        .max_width(384)
        .min_width(0)
        .font_size(12)
        .margin_top(8)
}
