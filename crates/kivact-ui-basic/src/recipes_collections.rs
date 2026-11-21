//! Role: Lyra recipes of the collections: the list, the data table, the static
//! table, the item list, the card, the toggle group and the button group, and the carousel.
//!
//! The looks of shadcn's data-table, item, card, toggle-group, button-group
//! and carousel recipes. The standard components draw them with
//! `kivact::ui::list`, `data_table`, `table`, `item_list`, `card`,
//! `toggle_group`, `button_group` and `carousel`.

use crate::color::{
    ACCENT, BACKGROUND, BORDER, CARD, CARD_FOREGROUND, FOREGROUND, INPUT, MUTED, MUTED_FOREGROUND,
};
use crate::recipe_parts::{theme_ring, toggle_style, wide_focus_ring};
use crate::variants::{
    CardHeaderBordered, CardSize, ItemMediaVariant, ItemSize, ItemVariant, ToggleVariant,
};
use kivact::custom::PartState;
use kivact::interaction::{ListItemState, TableHeaderState};
use kivact::motion::{Easing, Transition};
use kivact::style::{
    Align, Color, Cursor, FlexDirection, Justify, Overflow, Position, Style, Theme, TransitionProp,
    pct, style,
};
use kivact::ui::theme::{
    ButtonGroupMemberState, ButtonGroupState, ButtonGroupTheme, CardState, CardTheme,
    CarouselSurfaceState, CarouselTheme, DataTableCellState, DataTableRowState, DataTableTheme,
    ItemListTheme, ItemRowState, ListTheme, Look, TableTheme, ToggleGroupItemState,
    ToggleGroupState, ToggleGroupTheme,
};
use kivact::view::{Orientation, ReadingDirection};

/// `inline-flex w-fit`-like list rows: 32 px tall, 8 px inset, and the accent
/// behind a hovered or selected row.
fn list_item(_: &Theme, _: &Look<ListItemState>, base: Style) -> Style {
    base.padding_x(8.0)
        .min_height(32.0)
        .border_radius(6.0)
        .hover(style().background(ACCENT))
        .selected(style().background(ACCENT))
}

/// `text-xs`, muted.
fn list_description(_: &Theme, _: &Look<ListItemState>, _base: Style) -> Style {
    style().font_size(12.0).color(MUTED_FOREGROUND)
}

/// `rounded-md border`, the grid's frame.
fn data_table_frame(_: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.border_color(BORDER).border_radius(6.0)
}

/// `py-4` around the toolbar, with no inset at the sides.
fn data_table_toolbar(_: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.padding_x(0.0).padding_y(16.0)
}

/// `py-4` around the footer, with no inset at the sides.
fn data_table_footer(_: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.padding_x(0.0).padding_y(16.0)
}

/// `h-10 px-2 font-medium`, in the foreground over a bordered base.
fn data_table_header(_: &Theme, _: &Look<TableHeaderState>, base: Style) -> Style {
    base.min_height(40.0)
        .padding_x(8.0)
        .font_weight(500)
        .color(FOREGROUND)
        .border_color(BORDER)
}

/// `border-b hover:bg-muted/50 data-[state=selected]:bg-muted`.
fn data_table_row(_: &Theme, _: &Look<DataTableRowState>, base: Style) -> Style {
    base.border_color(BORDER)
        .hover(style().background(MUTED.with_alpha(0.5)))
        .selected(style().background(MUTED))
}

/// `p-2`, in the foreground.
fn data_table_cell(_: &Theme, _: &Look<DataTableCellState>, base: Style) -> Style {
    base.padding(8.0).color(FOREGROUND)
}

/// The static table: a full-width column of 12 px text that clips its rows.
fn table_root(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .width(pct(100.0))
        .font_size(12.0)
        .overflow(Overflow::Hidden)
}

fn table_header(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style().flex_direction(FlexDirection::Column)
}

fn table_body(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .margin_bottom(-1.0)
}

/// `border-t bg-muted/50 font-medium`.
fn table_footer(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .margin_bottom(-1.0)
        .border_top(1.0, BORDER)
        .background(MUTED.with_alpha(0.5))
        .font_weight(500)
}

/// `border-b hover:bg-muted/50 data-[state=selected]:bg-muted`.
fn table_row(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .width(pct(100.0))
        .border_bottom(1.0, BORDER)
        .hover(style().background(MUTED.with_alpha(0.5)))
        .selected(style().background(MUTED))
}

/// `h-10 px-2 font-medium`.
fn table_head(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .padding_x(8.0)
        .min_width(0.0)
        .text_wrap(false)
        .color(FOREGROUND)
        .height(40.0)
        .font_weight(500)
}

/// `p-2`.
fn table_cell(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .padding(8.0)
        .min_width(0.0)
        .text_wrap(false)
        .color(FOREGROUND)
}

/// The gap of a card and its padding: 16 px, or 12 px in the small size.
fn card_spacing(s: &Look<CardState>) -> f64 {
    match s.variant::<CardSize>().unwrap_or_default() {
        CardSize::Default => 16.0,
        CardSize::Sm => 12.0,
    }
}

/// `rounded-none` card with its ring and its padding, which a media and a footer
/// take away on their side.
fn card_root(_: &Theme, s: &Look<CardState>, _base: Style) -> Style {
    let spacing = card_spacing(s);
    style()
        .flex_direction(FlexDirection::Column)
        .gap(spacing)
        .padding_top(if s.has_media { 0.0 } else { spacing })
        .padding_bottom(if s.has_footer { 0.0 } else { spacing })
        .border_radius(0.0)
        .background(CARD)
        .color(CARD_FOREGROUND)
        .font_size(12.0)
        .outline(1.0, FOREGROUND.with_alpha(0.1))
        .overflow(Overflow::Hidden)
}

/// The header: a row of the card's padding, with a rule under it when bordered.
fn card_header(_: &Theme, s: &Look<CardState>, _base: Style) -> Style {
    let spacing = card_spacing(s);
    let base = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Start)
        .gap(8.0)
        .padding_x(spacing);
    if s.variant::<CardHeaderBordered>()
        .is_some_and(|bordered| bordered.0)
    {
        base.padding_bottom(spacing).border_bottom(1.0, BORDER)
    } else {
        base
    }
}

fn card_media(_: &Theme, _: &Look<CardState>, _base: Style) -> Style {
    style().flex_direction(FlexDirection::Column)
}

fn card_content(_: &Theme, s: &Look<CardState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .padding_x(card_spacing(s))
        .line_height(1.625)
}

/// The footer: a row of the card's padding under a rule.
fn card_footer(_: &Theme, s: &Look<CardState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .padding(card_spacing(s))
        .border_top(1.0, BORDER)
}

/// The column of an item list: the rows 16 px apart.
fn item_list_root(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .width(pct(100.0))
        .gap(16.0)
}

/// The item row: a bordered, muted or plain row, pointed at when it acts.
fn item_row(_: &Theme, s: &Look<ItemRowState>, _base: Style) -> Style {
    let (fill, line) = match s.variant::<ItemVariant>().unwrap_or_default() {
        ItemVariant::Default => (Color::TRANSPARENT, Color::TRANSPARENT),
        ItemVariant::Outline => (Color::TRANSPARENT, BORDER),
        ItemVariant::Muted => (MUTED.with_alpha(0.5), Color::TRANSPARENT),
    };
    let base = style()
        .flex_direction(FlexDirection::Column)
        .width(pct(100.0))
        .gap(10.0)
        .padding_x(12.0)
        .padding_y(10.0)
        .border(1.0, line)
        .border_radius(0.0)
        .background(fill)
        .color(FOREGROUND)
        .font_size(12.0);
    if s.actionable {
        base.cursor(Cursor::Pointer)
            .hover(style().background(MUTED))
            .focus_visible(wide_focus_ring())
    } else {
        base
    }
}

/// The row of an item's media, content and actions, reversed when right-to-left.
fn item_body(_: &Theme, _: &Look<ItemRowState>, _base: Style) -> Style {
    style()
        .flex_direction(if kivact::view::use_direction() == ReadingDirection::Rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .align_items(Align::Center)
        .gap(10.0)
        .width(pct(100.0))
}

fn item_content(_: &Theme, _: &Look<ItemRowState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .flex_grow(1.0)
        .gap(4.0)
        .min_width(0.0)
}

fn item_title(_: &Theme, _: &Look<ItemRowState>, _base: Style) -> Style {
    style().font_size(12.0).font_weight(500).single_line()
}

fn item_description(_: &Theme, _: &Look<ItemRowState>, _base: Style) -> Style {
    style()
        .line_height(1.625)
        .line_clamp(2)
        .color(MUTED_FOREGROUND)
}

/// The media box: centred, and 32 or 40 px square for an image.
fn item_media(_: &Theme, s: &Look<ItemRowState>, _base: Style) -> Style {
    let base = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .flex_shrink(0.0)
        .gap(8.0);
    let base = if s.has_description {
        base.align_self(Align::Start).translate(0.0, 2.0)
    } else {
        base
    };
    if s.variant::<ItemMediaVariant>() == Some(ItemMediaVariant::Image) {
        let side = if s.variant::<ItemSize>().unwrap_or_default() == ItemSize::Sm {
            32.0
        } else {
            40.0
        };
        base.size(side, side).overflow(Overflow::Hidden)
    } else {
        base
    }
}

fn item_actions(_: &Theme, _: &Look<ItemRowState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(8.0)
        .flex_shrink(0.0)
}

/// The header and footer band of an item: a row spread between its ends.
fn item_band(_: &Theme, _: &Look<ItemRowState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .width(pct(100.0))
        .align_items(Align::Center)
        .justify_content(Justify::SpaceBetween)
        .gap(8.0)
}

/// The row of a toggle group: a column when vertical, a row reversed when
/// right-to-left, with the group's spacing between the items.
fn toggle_group_root(_: &Theme, s: &Look<ToggleGroupState>, _base: Style) -> Style {
    let vertical = s.orientation == Orientation::Vertical;
    style()
        .flex_direction(if vertical {
            FlexDirection::Column
        } else if s.rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .align_items(if vertical {
            Align::Stretch
        } else {
            Align::Center
        })
        .gap(s.spacing.unwrap_or(2.0) * 4.0)
        .flex_shrink(0.0)
        .align_self(Align::Start)
}

/// A toggle of the group: a joined outline item drops the border it shares
/// with the one before it.
fn toggle_group_item(_: &Theme, s: &Look<ToggleGroupItemState>, _base: Style) -> Style {
    let variant = s.variant::<ToggleVariant>().unwrap_or_default();
    let joined = s.layout.spacing == Some(0.0);
    let base = toggle_style(
        variant == ToggleVariant::Outline,
        s.toggle.size,
        s.toggle.pressed,
        s.toggle.leading_icon,
        joined,
    );
    if joined && variant == ToggleVariant::Outline && !s.first {
        if s.layout.orientation == Orientation::Vertical {
            base.border_top(0.0, Color::TRANSPARENT)
        } else if s.layout.rtl {
            base.border_right(0.0, Color::TRANSPARENT)
        } else {
            base.border_left(0.0, Color::TRANSPARENT)
        }
    } else {
        base
    }
}

/// The row of a button group: members in a column when vertical, reversed
/// when right-to-left, with square corners.
fn button_group_root(_: &Theme, s: &Look<ButtonGroupState>, _base: Style) -> Style {
    style()
        .flex_direction(if s.orientation == Orientation::Vertical {
            FlexDirection::Column
        } else if s.rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .align_items(Align::Stretch)
        .border_radius(0.0)
}

/// A member after the first pulls its edge over the one before it by a pixel.
fn button_group_member(_: &Theme, s: &Look<ButtonGroupMemberState>, _base: Style) -> Style {
    let base = style().flex_shrink(0.0);
    if s.first {
        base
    } else if s.layout.orientation == Orientation::Vertical {
        base.margin_top(-1.0)
    } else if s.layout.rtl {
        base.margin_right(-1.0)
    } else {
        base.margin_left(-1.0)
    }
}

/// The rule between two members: 1 px, across the group.
fn button_group_separator(_: &Theme, s: &Look<ButtonGroupState>, _base: Style) -> Style {
    let base = style()
        .flex_shrink(0.0)
        .align_self(Align::Stretch)
        .background(INPUT);
    if s.orientation == Orientation::Vertical {
        base.height(1.0).margin_x(1.0)
    } else {
        base.width(1.0).margin_y(1.0)
    }
}

/// `bg-muted px-2.5 text-xs font-medium`: a text cell of the group.
fn button_group_text(_: &Theme, _: &Look<ButtonGroupState>, _base: Style) -> Style {
    style()
        .align_items(Align::Center)
        .gap(8.0)
        .padding_x(10.0)
        .border(1.0, BORDER)
        .border_radius(0.0)
        .background(MUTED)
        .color(FOREGROUND)
        .font_size(12.0)
        .font_weight(500)
        .text_wrap(false)
}

fn carousel_root(t: &Theme, _: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    style()
        .position(Position::Relative)
        .flex_direction(FlexDirection::Column)
        .focus(theme_ring(t))
}

/// `gap-4` between the slides, gliding on a 300 ms cubic-bezier.
fn carousel_track(_: &Theme, _: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    style().gap(16.0).transition(
        TransitionProp::All,
        Transition::tween(300.0, Easing::CubicBezier(0.4, 0.0, 0.2, 1.0)),
    )
}

fn carousel_viewport(_: &Theme, _: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    style().flex_direction(FlexDirection::Column)
}

fn carousel_slide(_: &Theme, _: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    style()
}

/// `absolute size-7 border bg-background`, beside the track.
fn carousel_control(_: &Theme, s: &Look<CarouselSurfaceState>, previous: bool) -> Style {
    let base = style()
        .position(Position::Absolute)
        .size(28.0, 28.0)
        .min_width(28.0)
        .padding(0.0)
        .background(BACKGROUND)
        .color(FOREGROUND)
        .border(1.0, INPUT)
        .hover(style().background(MUTED).color(FOREGROUND));
    if s.orientation == Orientation::Vertical {
        if previous {
            base.top(-48.0).left(pct(50.0) - 14.0)
        } else {
            base.bottom(-48.0).left(pct(50.0) - 14.0)
        }
    } else if previous != (s.direction == ReadingDirection::Rtl) {
        base.left(-48.0).top(pct(50.0) - 14.0)
    } else {
        base.right(-48.0).top(pct(50.0) - 14.0)
    }
}

fn carousel_previous(t: &Theme, s: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    carousel_control(t, s, true)
}

fn carousel_next(t: &Theme, s: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    carousel_control(t, s, false)
}

/// Installs the collection parts of the kit.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            ListTheme::default()
                .item(list_item)
                .description(list_description),
        )
        .with(
            DataTableTheme::default()
                .frame(data_table_frame)
                .toolbar(data_table_toolbar)
                .footer(data_table_footer)
                .header(data_table_header)
                .row(data_table_row)
                .cell(data_table_cell),
        )
        .with(
            TableTheme::default()
                .root(table_root)
                .header(table_header)
                .body(table_body)
                .footer(table_footer)
                .row(table_row)
                .head(table_head)
                .cell(table_cell),
        )
        .with(
            ItemListTheme::default()
                .root(item_list_root)
                .item(item_row)
                .body(item_body)
                .content(item_content)
                .title(item_title)
                .description(item_description)
                .media(item_media)
                .actions(item_actions)
                .header(item_band)
                .footer(item_band),
        )
        .with(
            CardTheme::default()
                .root(card_root)
                .header(card_header)
                .media(card_media)
                .content(card_content)
                .footer(card_footer),
        )
        .with(
            ToggleGroupTheme::default()
                .root(toggle_group_root)
                .item(toggle_group_item),
        )
        .with(
            ButtonGroupTheme::default()
                .root(button_group_root)
                .member(button_group_member)
                .separator(button_group_separator)
                .text(button_group_text),
        )
        .with(
            CarouselTheme::default()
                .root(carousel_root)
                .viewport(carousel_viewport)
                .track(carousel_track)
                .slide(carousel_slide)
                .previous(carousel_previous)
                .next(carousel_next),
        )
}
