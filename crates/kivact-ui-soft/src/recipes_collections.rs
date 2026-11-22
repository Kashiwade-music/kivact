//! Role: Install Astryx Neutral's recipes of the collection group: lists, trees, item
//! lists and cards, and the table, group and carousel recipes.
use crate::color as c;
use crate::metrics as m;
use crate::motion::{FAST, FAST_MIN};
use crate::radius as r;
use crate::recipe_parts::{body_text, fast, ring, supporting_text, tween};
use crate::shadows;
use crate::space as s;
use crate::variants::{
    CardPadding, CardVariant, Elevation, ItemAlign, ItemDensity, ItemHighlighted, ItemLayout,
    ListDividers, ListEdge, TreeGuides,
};
use kivact::custom::PartState;
use kivact::draw::{LineCap, Stroke};
use kivact::geometry::{Point, Rect};
use kivact::interaction::ListItemState;
use kivact::style::{
    Align, Color, Cursor, FlexDirection, Justify, Position, Shadow, Style, Theme, ThemeMode, style,
};
use kivact::ui::ItemListMarker;
use kivact::ui::theme::{
    CardState, CardTheme, ItemListTheme, ItemRowState, ListTheme, Look, Selectable,
    TreeRowRecipeState, TreeTheme,
};
use kivact::view::{Orientation, ReadingDirection, use_direction};

/// The column of a list: its rows, a small gap apart.
fn list_root(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style().gap(s::SPACE_S0_5)
}

/// One row of a list: its label, with the hover, pressed and selected fills.
fn list_item(_: &Theme, look: &Look<ListItemState>, _base: Style) -> Style {
    let disabled = look.input.is_disabled();
    let item = body_text(style())
        .color(if disabled {
            c::TEXT_DISABLED
        } else {
            c::TEXT_PRIMARY
        })
        .align_items(Align::Center)
        .gap(s::SPACE_S2)
        .padding(s::SPACE_S2)
        .border_radius(r::RADIUS_ELEMENT)
        .selected(style().background(c::ACCENT_MUTED))
        .focus_visible(ring(c::FOCUS_RING));
    let item = if disabled {
        item
    } else {
        item.hover(style().background(c::OVERLAY_HOVER))
            .active(style().background(c::OVERLAY_PRESSED))
    };
    tween(item, FAST_MIN)
}

/// The secondary line of a list row: the supporting role.
fn list_description(_: &Theme, _: &Look<ListItemState>, _base: Style) -> Style {
    supporting_text(style())
}

/// The column of a tree: its visible rows, a small gap apart.
fn tree_root(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .gap(s::SPACE_S0_5)
}

/// One row of a tree: the indent of its level, the body role, and the guides of its ancestors.
fn tree_item(_: &Theme, row: &Look<TreeRowRecipeState>, _base: Style) -> Style {
    let density = row.variant::<ItemDensity>().unwrap_or_default();
    let guides = row.variant::<TreeGuides>().is_none_or(|guides| guides.0);
    let rtl = use_direction() == ReadingDirection::Rtl;
    let indent = 8 + row.level.saturating_sub(1) * 16;
    let look = body_text(style())
        .enter_size(Orientation::Vertical)
        .exit_size(Orientation::Vertical)
        .flex_direction(if rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .position(Position::Relative)
        .align_items(Align::Center)
        .min_width(0)
        .gap(8)
        .padding_y(density.padding())
        .padding_left(if rtl { 8 } else { indent })
        .padding_right(if rtl { indent } else { 8 })
        .border_radius(r::RADIUS_ELEMENT)
        .hover(style().background(c::OVERLAY_HOVER))
        .selected(style().background(c::ACCENT_MUTED))
        .focus_visible(ring(c::FOCUS_RING));
    if !guides || row.ancestor_last.is_empty() {
        return look;
    }
    let mut lines: Vec<bool> = row
        .ancestor_last
        .iter()
        .copied()
        .skip(1)
        .map(|last| !last)
        .collect();
    lines.push(!row.is_last);
    let guide = c::BORDER_EMPHASIZED;
    look.paint(move |draw, size| {
        for (ancestor, extend) in lines.iter().enumerate() {
            let inset = 16.0 + ancestor as f64 * 16.0;
            let x = if rtl {
                size.width - inset - 0.5
            } else {
                inset - 0.5
            };
            let height = if *extend {
                size.height + 2.0
            } else {
                size.height
            };
            draw.rounded_rect(Rect::new(x, 0.0, 1.0, height), 0.5, guide);
        }
    })
}

/// The expand marker of a tree row: a chevron on a branch, turned down when it is expanded.
fn tree_marker(_: &Theme, row: &Look<TreeRowRecipeState>, _base: Style) -> Style {
    let rtl = use_direction() == ReadingDirection::Rtl;
    let branch = row.has_branches && !row.custom_marker;
    let angle = if row.branch.is_expanded() {
        if rtl { -90.0 } else { 90.0 }
    } else {
        0.0
    };
    let ink = c::ICON_SECONDARY;
    let marker = style()
        .size(16, 16)
        .flex_shrink(0.0)
        .font_size(if row.custom_marker { 14.0 } else { 0.0 })
        .color(ink)
        .rotate(angle);
    tween(marker, FAST).paint(move |draw, _| {
        if branch {
            let x = if rtl {
                [10.0, 6.0, 10.0]
            } else {
                [6.0, 10.0, 6.0]
            };
            draw.polyline(
                [
                    Point::new(x[0], 4.0),
                    Point::new(x[1], 8.0),
                    Point::new(x[2], 12.0),
                ],
                &Stroke::new(4.0 / 3.0)
                    .cap(LineCap::Round)
                    .join(kivact::draw::LineJoin::Round),
                ink,
            );
        }
    })
}

/// The column of an item list: its rows, with no gap when they have dividers.
fn item_root(_: &Theme, look: &Look<PartState>, _base: Style) -> Style {
    let gap = if look.variant::<ListDividers>().is_some_and(|d| d.0) {
        s::SPACE_S0
    } else {
        s::SPACE_S0_5
    };
    style().flex_direction(FlexDirection::Column).gap(gap)
}

/// The header of an item list: a column that takes its width from the list.
fn list_header(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style().flex_direction(FlexDirection::Column).min_width(0)
}

/// One row of an item list: its padding, its divider, its highlight and its hover and press fills.
fn item(_: &Theme, look: &Look<ItemRowState>, _base: Style) -> Style {
    let edge = look.variant::<ListEdge>().unwrap_or_default();
    let highlighted = look.variant::<ItemHighlighted>().is_some_and(|h| h.0);
    let selected = if look.actionable && look.input.is_pressed() {
        c::OVERLAY_PRESSED
    } else if look.actionable && look.input.is_hovered() {
        c::OVERLAY_HOVER
    } else {
        c::ACCENT_MUTED
    };
    let row = style()
        .flex_direction(FlexDirection::Column)
        .min_width(0)
        .padding_x(if edge == ListEdge::Flush { 0 } else { 12 })
        .padding_y(look.variant::<ItemDensity>().unwrap_or_default().padding())
        .background(if highlighted {
            c::OVERLAY_HOVER
        } else {
            Color::TRANSPARENT
        })
        .selected(style().background(selected));
    let row = if edge == ListEdge::Bleed {
        row.border_radius(0.0)
    } else {
        row.border_radius(r::RADIUS_ELEMENT)
    };
    let row = if look.variant::<ListDividers>().is_some_and(|d| d.0)
        && look.position.is_some_and(|p| p.index + 1 < p.count)
    {
        row.border_bottom(m::BORDER_WIDTH, c::BORDER)
    } else {
        row
    };
    if look.actionable {
        tween(
            row.cursor(Cursor::Pointer)
                .hover(style().background(c::OVERLAY_HOVER))
                .active(style().background(c::OVERLAY_PRESSED))
                .focus_visible(ring(c::FOCUS_RING)),
            FAST,
        )
    } else {
        row
    }
}

/// The body of a row: its icon and its content in a row, aligned by the item's alignment.
fn body(_: &Theme, look: &Look<ItemRowState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .min_width(0)
        .gap(8)
        .align_items(
            if look.variant::<ItemAlign>().unwrap_or_default() == ItemAlign::Center {
                Align::Center
            } else {
                Align::Start
            },
        )
}

/// The content of a row: its label and description, in a column or on one line.
fn content(_: &Theme, look: &Look<ItemRowState>, _base: Style) -> Style {
    let inline = look.variant::<ItemLayout>().unwrap_or_default() == ItemLayout::Inline;
    style()
        .flex_grow(1.0)
        .min_width(0)
        .flex_direction(if inline {
            FlexDirection::Row
        } else {
            FlexDirection::Column
        })
        .gap(if inline { 8 } else { 0 })
}

/// The label of a row: the body role in the primary color.
fn title(_: &Theme, _: &Look<ItemRowState>, _base: Style) -> Style {
    body_text(style())
}

/// The description of a row: the supporting role in the secondary color.
fn description(_: &Theme, _: &Look<ItemRowState>, _base: Style) -> Style {
    supporting_text(style())
}

/// The media of a row: an icon or a thumbnail that does not shrink.
fn media(_: &Theme, _: &Look<ItemRowState>, _base: Style) -> Style {
    style().flex_shrink(0.0).min_width(0)
}

/// The marker of a row: the bullet or the number, right-aligned in its own column.
fn marker(_: &Theme, look: &Look<ItemRowState>, _base: Style) -> Style {
    let base = style()
        .flex_direction(FlexDirection::Row)
        .justify_content(Justify::End)
        .flex_shrink(0.0)
        .min_width(0);
    match look.marker {
        ItemListMarker::None => base,
        ItemListMarker::Decimal => {
            let count = look.position.map_or(1, |p| p.count);
            let digits = (u64::from(look.start) + count.saturating_sub(1) as u64)
                .to_string()
                .len();
            base.width(digits as f64 * 8.0 + 8.0)
        }
        _ => base.width(16.0),
    }
}

/// The fills of a card for its variant: the surface and the ring of a selected card.
fn card_palette(variant: CardVariant) -> (Color, Color) {
    match variant {
        CardVariant::Default => (c::BG_CARD, c::ACCENT),
        CardVariant::Transparent => (Color::TRANSPARENT, c::ACCENT),
        CardVariant::Muted => (c::BG_MUTED, c::ACCENT),
        CardVariant::Blue => (c::HUE_BLUE_BG, c::HUE_BLUE_BORDER),
        CardVariant::Cyan => (c::HUE_CYAN_BG, c::HUE_CYAN_BORDER),
        CardVariant::Gray => (c::HUE_GRAY_BG, c::HUE_GRAY_BORDER),
        CardVariant::Green => (c::HUE_GREEN_BG, c::HUE_GREEN_BORDER),
        CardVariant::Orange => (c::HUE_ORANGE_BG, c::HUE_ORANGE_BORDER),
        CardVariant::Pink => (c::HUE_PINK_BG, c::HUE_PINK_BORDER),
        CardVariant::Purple => (c::HUE_PURPLE_BG, c::HUE_PURPLE_BORDER),
        CardVariant::Red => (c::HUE_RED_BG, c::HUE_RED_BORDER),
        CardVariant::Teal => (c::HUE_TEAL_BG, c::HUE_TEAL_BORDER),
        CardVariant::Yellow => (c::HUE_YELLOW_BG, c::HUE_YELLOW_BORDER),
    }
}

/// The surface of a card: its fill, its border when it is the bordered default, its selection ring, and its elevation.
fn card_root(theme: &Theme, look: &Look<CardState>, _base: Style) -> Style {
    let variant = look.variant::<CardVariant>().unwrap_or_default();
    let bordered = variant == CardVariant::Default;
    let (fill, ring_color) = card_palette(variant);
    let selection = look.selection();
    let selected = selection == Selectable::Selected;
    let mut base = style()
        .flex_direction(FlexDirection::Column)
        .position(Position::Relative)
        .min_width(0)
        .border_radius(r::RADIUS_CONTAINER)
        .clip(true)
        .background(fill);
    if bordered {
        base = base.border(
            m::BORDER_WIDTH,
            if selected { ring_color } else { c::BORDER },
        );
    }
    if selection != Selectable::NotSelectable {
        base = base.shadow(
            Shadow::new(
                0,
                0,
                0,
                if selected {
                    ring_color
                } else {
                    Color::TRANSPARENT
                },
            )
            .spread(2)
            .inset(),
        );
    }
    if look.is_interactive() {
        base = fast(base.focus_visible(ring(c::ACCENT)));
        if look.disabled {
            base = base.opacity(m::DISABLED_OPACITY);
        } else if selection == Selectable::NotSelectable && bordered {
            base = base.hover(style().border_color(c::BORDER_EMPHASIZED));
        }
    }
    shadows::elevate(
        base,
        look.variant::<Elevation>().unwrap_or_default(),
        theme.mode() == ThemeMode::Dark,
    )
}

/// The card's content inset: the card padding, less the border of the bordered default.
fn card_padding(theme: &Theme, look: &Look<CardState>) -> f64 {
    let variant = look.variant::<CardVariant>().unwrap_or_default();
    let logical = look.variant::<CardPadding>().map_or_else(
        || theme.get(&s::SPACE_CARD_PADDING_VAR),
        |padding| padding.0.px(),
    );
    let border = if variant == CardVariant::Default {
        m::BORDER_WIDTH
    } else {
        0.0
    };
    (logical - border).max(0.0)
}

/// The content of a card: a column with the card's inset.
fn card_content(theme: &Theme, look: &Look<CardState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .padding(card_padding(theme, look))
        .min_width(0)
}

/// A card region that takes no style of its own: the header, the media and the footer.
fn card_empty(_: &Theme, _: &Look<CardState>, _base: Style) -> Style {
    style()
}

/// The hover and pressed fill over a card that has an input.
fn card_overlay(_: &Theme, look: &Look<CardState>, _base: Style) -> Style {
    let color =
        look.input()
            .filter(|input| !input.is_disabled())
            .map_or(Color::TRANSPARENT, |input| {
                if input.is_pressed() {
                    c::OVERLAY_PRESSED
                } else if input.is_hovered() {
                    c::OVERLAY_HOVER
                } else {
                    Color::TRANSPARENT
                }
            });
    fast(style().border_radius(r::RADIUS_CONTAINER).background(color))
}

/// Install the kit's recipes of the collection group: lists, trees, item lists and cards, then the table, group and carousel recipes.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme.with(
        ListTheme::default()
            .root(list_root)
            .item(list_item)
            .description(list_description),
    );
    let theme = theme.with(
        TreeTheme::default()
            .root(tree_root)
            .item(tree_item)
            .marker(tree_marker),
    );
    let theme = theme.with(
        ItemListTheme::default()
            .root(item_root)
            .list_header(list_header)
            .item(item)
            .body(body)
            .content(content)
            .title(title)
            .description(description)
            .media(media)
            .actions(media)
            .marker(marker),
    );
    let theme = theme.with(
        CardTheme::default()
            .root(card_root)
            .header(card_empty)
            .media(card_empty)
            .content(card_content)
            .footer(card_empty)
            .overlay(card_overlay),
    );
    let theme = crate::recipes_collections_table::install(theme);
    let theme = crate::recipes_groups::install(theme);
    crate::recipes_carousel::install(theme)
}
