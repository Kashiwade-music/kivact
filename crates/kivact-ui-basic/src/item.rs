//! Role: Basic item-row visual variants and canonical list recipes.
use crate::Tokens;
use crate::theme_access::wide_focus_ring;
use kivact::custom::PartState;
use kivact::state::{IntoValue, Value};
use kivact::style::{Align, Color, Cursor, Direction, Justify, Overflow, Style, Theme, pct, style};
use kivact::ui::{ItemListRecipes, ItemRowState, Look};
/// The frame of an standard item row (item.tsx `variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ItemVariant {
    /// Transparent border.
    #[default]
    Default,
    /// Bordered.
    Outline,
    /// Muted fill.
    Muted,
}

/// The density of an standard item row (item.tsx `size`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ItemSize {
    /// 12 px / 10 px padding.
    #[default]
    Default,
    /// The same padding with a smaller image.
    Sm,
}

/// The frame of the media slot of an standard item row (item.tsx media `variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ItemMediaVariant {
    /// Bare content.
    #[default]
    Default,
    /// An icon.
    Icon,
    /// A 40 px (32 px for `Sm`) clipped picture.
    Image,
}

impl kivact::ui::Variant for ItemVariant {}
impl kivact::ui::Variant for ItemSize {}
impl kivact::ui::Variant for ItemMediaVariant {}
kivact::state::constant_value!(ItemSize);
/// Reactive visual size for a standard item row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemLook {
    size: Value<ItemSize>,
}
impl Default for ItemLook {
    fn default() -> Self {
        Self {
            size: ItemSize::Default.into_value(),
        }
    }
}
impl kivact::ui::Variant for ItemLook {}
impl ItemLook {
    /// Set constant or reactive media density.
    pub fn size(mut self, size: impl IntoValue<ItemSize>) -> Self {
        self.size = size.into_value();
        self
    }
}
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
            if kivact::view::use_direction().get() == kivact::view::ReadingDirection::Rtl {
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
