//! Role: Basic visual card size variants and canonical card slot recipes.
use crate::Tokens;
use kivact::state::{IntoValue, Value};
use kivact::style::{Align, Direction, Overflow, Style, Theme, style};
use kivact::ui::{CardRecipes, CardState, Look};
/// Spacing of a standard card.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CardSize {
    #[default]
    Default,
    Sm,
}
impl kivact::ui::Variant for CardSize {}
kivact::state::constant_value!(CardSize);
/// Reactive spacing and independent header border of a standard card.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardLook {
    size: Value<CardSize>,
    header_bordered: Value<bool>,
}
impl Default for CardLook {
    fn default() -> Self {
        Self {
            size: CardSize::Default.into_value(),
            header_bordered: false.into_value(),
        }
    }
}
impl kivact::ui::Variant for CardLook {}
impl CardLook {
    /// Use constant or reactive card spacing.
    pub fn size(mut self, size: impl IntoValue<CardSize>) -> Self {
        self.size = size.into_value();
        self
    }
    /// Draw the header rule while true.
    pub fn header_bordered(mut self, bordered: impl IntoValue<bool>) -> Self {
        self.header_bordered = bordered.into_value();
        self
    }
}
fn spacing(s: &Look<CardState>) -> f64 {
    match s
        .variant::<CardLook>()
        .map(|look| look.size.get())
        .unwrap_or_else(|| s.variant::<CardSize>().copied().unwrap_or_default())
    {
        CardSize::Default => 16.0,
        CardSize::Sm => 12.0,
    }
}
pub(crate) fn recipes(r: CardRecipes) -> CardRecipes {
    r.root(|_| root)
        .header(|_| header)
        .media(|_| media)
        .content(|_| content)
        .footer(|_| footer)
}
fn root(t: &Theme, s: &Look<CardState>) -> Style {
    let p = t.ext::<Tokens>();
    let spacing = spacing(s);
    style()
        .direction(Direction::Column)
        .gap(spacing)
        .padding_top(if s.has_media { 0.0 } else { spacing })
        .padding_bottom(if s.has_footer { 0.0 } else { spacing })
        .radius(0)
        .background(p.card)
        .color(p.card_foreground)
        .font_size(12)
        .ring(1, p.foreground.with_alpha(0.1))
        .overflow(Overflow::Hidden)
}
fn header(t: &Theme, s: &Look<CardState>) -> Style {
    let spacing = spacing(s);
    let base = style()
        .direction(Direction::Row)
        .align_items(Align::Start)
        .gap(8)
        .padding_x(spacing);
    if s.variant::<CardLook>()
        .is_some_and(|look| look.header_bordered.get())
    {
        base.padding_bottom(spacing)
            .border_bottom(1, t.ext::<Tokens>().border)
    } else {
        base
    }
}
fn media(_: &Theme, _: &Look<CardState>) -> Style {
    style().direction(Direction::Column)
}
fn content(_: &Theme, s: &Look<CardState>) -> Style {
    style()
        .direction(Direction::Column)
        .padding_x(spacing(s))
        .line_height(1.625)
}
fn footer(t: &Theme, s: &Look<CardState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .padding(spacing(s))
        .border_top(1, t.ext::<Tokens>().border)
}
