//! Role: Style canonical table rows and cells with Soft paint choices.
use crate::{ItemDensity, Tokens};
use kivact::custom::PartState;
use kivact::state::{IntoValue, Value};
use kivact::style::{Align, Direction, Style, Theme, pct, style};
use kivact::ui::{Look, TableRecipes};

/// Independent visual choices for a static table row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableRowAppearance {
    /// Whether row separators are painted.
    pub dividers: Value<bool>,
    /// Whether hovered rows receive hover paint.
    pub hover: Value<bool>,
    /// Whether alternating rows receive stripe paint.
    pub striped: Value<bool>,
}
impl TableRowAppearance {
    /// Create reactive row separator, hover and stripe choices.
    pub fn new(
        dividers: impl IntoValue<bool>,
        hover: impl IntoValue<bool>,
        striped: impl IntoValue<bool>,
    ) -> Self {
        Self {
            dividers: dividers.into_value(),
            hover: hover.into_value(),
            striped: striped.into_value(),
        }
    }
}
impl kivact::ui::Variant for TableRowAppearance {}

/// Independent visual choices for a static header or data cell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableCellAppearance {
    /// Reactive cell padding density.
    pub density: Value<ItemDensity>,
    /// Reactive vertical alignment within the cell.
    pub vertical_align: Value<Align>,
    /// Whether the cell paints a separator.
    pub divider: Value<bool>,
}
impl TableCellAppearance {
    /// Create reactive cell padding, vertical alignment and separator choices.
    pub fn new(
        density: impl IntoValue<ItemDensity>,
        vertical_align: impl IntoValue<Align>,
        divider: impl IntoValue<bool>,
    ) -> Self {
        Self {
            density: density.into_value(),
            vertical_align: vertical_align.into_value(),
            divider: divider.into_value(),
        }
    }
}
impl kivact::ui::Variant for TableCellAppearance {}

pub(crate) fn table(r: TableRecipes) -> TableRecipes {
    r.root(|_| {
        |_: &Theme, _: &_| {
            style()
                .direction(Direction::Column)
                .width(pct(100))
                .min_width(0)
        }
    })
    .header(|_| |_: &Theme, _: &_| style().direction(Direction::Column))
    .body(|_| |_: &Theme, _: &_| style().direction(Direction::Column))
    .footer(|_| {
        |t: &Theme, _: &_| {
            style()
                .direction(Direction::Column)
                .background(t.ext::<Tokens>().colors.bg_muted)
        }
    })
    .row(|_| row)
    .head(|_| |t: &Theme, s: &_| cell(t, s).weight(600))
    .cell(|_| cell)
}
fn row(t: &Theme, s: &Look<PartState>) -> Style {
    let p = t.ext::<Tokens>();
    let appearance = s.variant::<TableRowAppearance>();
    let mut row = style()
        .direction(Direction::Row)
        .width(pct(100))
        .min_width(0)
        .selected(style().background(p.colors.accent_muted));
    if appearance.is_none_or(|v| v.dividers.get()) {
        row = row.border_bottom(1, p.colors.border);
    }
    if appearance.is_some_and(|v| v.hover.get()) {
        row = row.hover(style().background(p.colors.overlay_hover));
    }
    if appearance.is_some_and(|v| v.striped.get()) {
        row = row.background(p.colors.bg_muted);
    }
    row
}
fn cell(t: &Theme, s: &Look<PartState>) -> Style {
    let p = t.ext::<Tokens>();
    let appearance = s.variant::<TableCellAppearance>();
    let padding = appearance
        .map_or(ItemDensity::Balanced, |v| v.density.get())
        .padding();
    let mut cell = style()
        .direction(Direction::Row)
        .align_items(appearance.map_or(Align::Center, |v| v.vertical_align.get()))
        .min_width(0)
        .padding_y(padding)
        .padding_x(padding + 4)
        .font_size(14)
        .line_height(20.0 / 14.0)
        .color(p.colors.text_primary);
    if appearance.is_some_and(|v| v.divider.get()) {
        cell = cell.border_right(1, p.colors.border);
    }
    cell
}
