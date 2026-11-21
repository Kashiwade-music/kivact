//! Role: Basic item-row visual variants and canonical list recipes.
use crate::Tokens;
use crate::theme_access::wide_focus_ring;
use kivact::Prop;
use kivact::custom::PartState;
use kivact::state::{IntoValue, Value};
use kivact::style::{Align, Color, Cursor, Direction, Justify, Overflow, Style, Theme, pct, style};
use kivact::ui::{ItemListRecipes, ItemRowState, Look};
pub(crate) fn recipes(r: ItemListRecipes) -> ItemListRecipes {
    r.root(|_| root)
        .item(|_| item)
        .body(|_| body)
        .content(|_| content)
        .title(|_| title)
        .description(|_| description)
        .media(|_| media)
        .actions(|_| actions)
        .header(|_| band)
        .footer(|_| band)
}
fn root(_: &Theme, _: &Look<PartState>) -> Style {
    style().direction(Direction::Column).width(pct(100)).gap(16)
}
fn item(t: &Theme, s: &Look<ItemRowState>) -> Style {
    let p = t.ext::<Tokens>();
    let (fill, line) = match s.variant::<ItemVariant>().copied().unwrap_or_default() {
        ItemVariant::Default => (Color::TRANSPARENT, Color::TRANSPARENT),
        ItemVariant::Outline => (Color::TRANSPARENT, p.border),
        ItemVariant::Muted => (p.muted.with_alpha(0.5), Color::TRANSPARENT),
    };
    let base = style()
        .direction(Direction::Column)
        .width(pct(100))
        .gap(10)
        .padding_x(12)
        .padding_y(10)
        .border(1, line)
        .radius(0)
        .background(fill)
        .color(p.foreground)
        .font_size(12);
    if s.actionable {
        base.cursor(Cursor::Pointer)
            .hover(style().background(p.muted))
            .focus_visible(wide_focus_ring(&p))
    } else {
        base
    }
}
fn body(_: &Theme, _: &Look<ItemRowState>) -> Style {
    style()
        .direction(
            if kivact::view::use_direction() == kivact::view::ReadingDirection::Rtl {
                Direction::RowReverse
            } else {
                Direction::Row
            },
        )
        .align_items(Align::Center)
        .gap(10)
        .width(pct(100))
}
fn content(_: &Theme, _: &Look<ItemRowState>) -> Style {
    style()
        .direction(Direction::Column)
        .grow(1)
        .gap(4)
        .min_width(0)
}
fn title(_: &Theme, _: &Look<ItemRowState>) -> Style {
    style().font_size(12).weight(500).single_line()
}
fn description(t: &Theme, _: &Look<ItemRowState>) -> Style {
    style()
        .line_height(1.625)
        .line_clamp(2)
        .color(t.ext::<Tokens>().muted_foreground)
}
fn media(_: &Theme, s: &Look<ItemRowState>) -> Style {
    let base = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .shrink(0)
        .gap(8);
    let base = if s.has_description {
        base.align_self(Align::Start).translate(0, 2)
    } else {
        base
    };
    if s.variant::<ItemMediaVariant>() == Some(&ItemMediaVariant::Image) {
        let size = s
            .variant::<ItemLook>()
            .map(|look| look.size.get())
            .unwrap_or_else(|| s.variant::<ItemSize>().copied().unwrap_or_default());
        let side = if size == ItemSize::Sm { 32 } else { 40 };
        base.size(side, side).overflow(Overflow::Hidden)
    } else {
        base
    }
}
fn actions(_: &Theme, _: &Look<ItemRowState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(8)
        .shrink(0)
}
fn band(_: &Theme, _: &Look<ItemRowState>) -> Style {
    style()
        .direction(Direction::Row)
        .width(pct(100))
        .align_items(Align::Center)
        .justify_content(Justify::SpaceBetween)
        .gap(8)
}
