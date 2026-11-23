//! Role: Lyra recipes of the overlay parts: select, menu, tooltip and hover card.
//!
//! The looks of select.tsx, dropdown-menu.tsx, tooltip.tsx and hover-card.tsx.
//! The menu bar and the use_context menu share the menu recipes: a menu of
//! [`MenuVariant::Bar`] has the menubar.tsx measures. The dialog, command
//! palette and combobox recipes are in the `recipes_overlays_*` modules,
//! installed from here.

use crate::color::{
    ACCENT, ACCENT_FOREGROUND, BACKGROUND, BORDER, DESTRUCTIVE, FOREGROUND, INPUT,
    MUTED_FOREGROUND, POPOVER_FOREGROUND,
};
use crate::recipe_parts::{
    ButtonLook, CHEVRON_DOWN, field_fill, list_row, option_row, popover_surface, shadow_lg,
    shadow_md, stroke_glyph, surface_style, trigger_style, with_focus,
};
use crate::variants::MenuVariant;
use kivact::custom::PartState;
use kivact::geometry::{Edge, Point};
use kivact::interaction::{ListItemState, MenuItemState, PopupState};
use kivact::style::{Align, FlexDirection, Justify, Position, Style, Theme, ThemeMode, pct, style};
use kivact::ui::ControlSize;
use kivact::ui::theme::{
    HoverCardSurfaceState, HoverCardTheme, Look, MenuTheme, SelectTheme, SelectTriggerState,
    TooltipArrowState, TooltipTheme,
};

/// The measures that differ between the dropdown, use_context and bar menus.
#[derive(Clone, Copy)]
struct MenuLook {
    /// The least width of the top level (`min-w-32`, `min-w-36`).
    min_width: f64,
    /// The least width of a submenu.
    sub_min_width: f64,
    /// The menu bar's rows: check marks at the start, margins around
    /// separators and a wider inset (`pl-8`).
    bar: bool,
}

const DROPDOWN: MenuLook = MenuLook {
    min_width: 128.0,
    sub_min_width: 96.0,
    bar: false,
};
/// use_context-menu.tsx's top level (`min-w-36`).
const CONTEXT_MIN_WIDTH: f64 = 144.0;
const BAR: MenuLook = MenuLook {
    min_width: 144.0,
    sub_min_width: 128.0,
    bar: true,
};

/// The measures of the kind of menu `look` belongs to.
fn kind<S>(look: &Look<S>) -> MenuLook {
    match look.variant::<MenuVariant>() {
        Some(MenuVariant::Bar) => BAR,
        None => DROPDOWN,
    }
}

/// `bg-popover text-popover-foreground ring-1 ring-foreground/10`, in a column.
fn menu_base() -> Style {
    popover_surface().flex_direction(FlexDirection::Column)
}

/// The select trigger: a bordered field, its text muted while it shows the
/// placeholder.
fn select_trigger(t: &Theme, s: &Look<SelectTriggerState>, _base: Style) -> Style {
    let dark = t.mode() == ThemeMode::Dark;
    let base = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::SpaceBetween)
        .gap(6.0)
        // `data-[size=default]:h-8 data-[size=sm]:h-7`, like input and button.
        .height(match s.size {
            ControlSize::ExtraSmall => 24.0,
            ControlSize::Small => 28.0,
            ControlSize::Medium => 32.0,
            ControlSize::Large => 36.0,
        })
        .padding_left(10.0)
        .padding_right(8.0)
        .border_radius(0.0)
        .border(1.0, INPUT)
        .background(field_fill(dark))
        // `data-placeholder:text-muted-foreground`.
        .color(if s.placeholder {
            MUTED_FOREGROUND
        } else {
            FOREGROUND
        })
        .font_size(12.0)
        .text_wrap(false);
    let base = if dark {
        base.hover(style().background(INPUT.with_alpha(0.5)))
    } else {
        base
    };
    with_focus(base, t)
}

/// The floating list: [`surface_style`] at least 144 px wide, 320 px high at most.
fn select_surface(_: &Theme, _: &Look<PopupState>, _base: Style) -> Style {
    surface_style().min_width(144.0).max_height(320.0)
}

fn select_option(_: &Theme, s: &Look<ListItemState>, _base: Style) -> Style {
    option_row(s.active)
}

/// The 16 px box of a `check` at a row's right, in the row's color.
fn select_check(_: &Theme, _: &Look<ListItemState>, _base: Style) -> Style {
    style()
        .position(Position::Absolute)
        .right(8.0)
        .top(pct(50.0) - 8.0)
        .size(16.0, 16.0)
}

/// The box of the trigger's `chevron-down` icon.
fn select_indicator(_: &Theme, _: &Look<PopupState>, _base: Style) -> Style {
    style()
        .size(16.0, 16.0)
        .flex_shrink(0.0)
        .color(MUTED_FOREGROUND)
}

fn select_description(_: &Theme, _: &Look<ListItemState>, _base: Style) -> Style {
    style().color(MUTED_FOREGROUND)
}

/// An outline button with a trailing `chevron-down`.
fn menu_trigger(t: &Theme, s: &Look<PopupState>, _base: Style) -> Style {
    trigger_style(
        t,
        ButtonLook::Outline,
        ControlSize::Medium,
        &s.input,
        Some(s.open),
    )
    .padding_right(30.0)
    .paint(move |d, size| {
        let at = Point::new(size.width - 24.0, (size.height - 16.0) / 2.0);
        stroke_glyph(d, at, 16.0, CHEVRON_DOWN, MUTED_FOREGROUND);
    })
}

/// `shadow-md` on the top level, `min-w-32` (`min-w-36` in a context menu).
fn menu_surface(_: &Theme, s: &Look<PopupState>, _base: Style) -> Style {
    shadow_md(menu_base()).min_width(kind(s).min_width)
}

fn menu_context_surface(_: &Theme, _: &Look<PopupState>, _base: Style) -> Style {
    shadow_md(menu_base()).min_width(CONTEXT_MIN_WIDTH)
}

fn menu_sub_surface(_: &Theme, s: &Look<PopupState>, _base: Style) -> Style {
    shadow_lg(menu_base()).min_width(kind(s).sub_min_width)
}

/// `style` for a part inside a row (its shortcut, mark or chevron) while
/// the row is lit: open, or (while enabled) hovered or focused, which the
/// part reads from the row, a style group.
fn when_lit(s: &Look<MenuItemState>, base: Style, lit: Style) -> Style {
    if s.is_expanded() {
        base.merge(&lit)
    } else if s.input.is_disabled() {
        base
    } else {
        base.group_hover(lit.clone()).group_focus(lit)
    }
}

/// `gap-2 py-2 px-2 text-xs`, `bg-accent text-accent-foreground` while
/// pointed at, focused or open; `text-destructive` with `bg-destructive/10`
/// for a destructive item. A check or radio item keeps room for its mark
/// at the end (`pr-8`), or at the start in the menu bar (`pl-8`).
fn item(t: &Theme, s: &Look<MenuItemState>, look: MenuLook) -> Style {
    let dark = t.mode() == ThemeMode::Dark;
    let (left, right) = match (s.check().is_some(), look.bar) {
        (true, true) => (32.0, 8.0),
        (true, false) => (8.0, 32.0),
        (false, _) => (8.0, 8.0),
    };
    let lit_style = if s.destructive {
        style()
            .background(DESTRUCTIVE.with_alpha(if dark { 0.2 } else { 0.1 }))
            .color(DESTRUCTIVE)
    } else {
        style().background(ACCENT).color(ACCENT_FOREGROUND)
    };
    // The row is the style group its shortcut, mark and chevron follow.
    let base = list_row(false)
        .group()
        .padding_left(left)
        .padding_right(right)
        .color(if s.destructive {
            DESTRUCTIVE
        } else {
            POPOVER_FOREGROUND
        })
        .hover(lit_style.clone())
        .focus(lit_style.clone());
    if s.is_expanded() {
        base.merge(&lit_style)
    } else {
        base
    }
}

fn menu_item(t: &Theme, s: &Look<MenuItemState>, _base: Style) -> Style {
    item(t, s, kind(s))
}

fn menu_separator(_: &Theme, s: &Look<PartState>, _base: Style) -> Style {
    let line = style().height(1.0).flex_shrink(0.0).background(BORDER);
    if kind(s).bar {
        line.margin_y(4.0)
    } else {
        line
    }
}

/// A heading: `px-2 py-2 text-xs text-muted-foreground`, with the inset box
/// before its text.
fn menu_label(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .gap(8.0)
        .padding(8.0)
        .font_size(12.0)
        .text_wrap(false)
        .color(MUTED_FOREGROUND)
}

/// `ml-auto text-xs tracking-widest`, muted unless its row is lit.
fn menu_shortcut(_: &Theme, s: &Look<MenuItemState>, _base: Style) -> Style {
    let lit = if s.destructive {
        DESTRUCTIVE
    } else {
        ACCENT_FOREGROUND
    };
    when_lit(
        s,
        crate::recipe_parts::menu_shortcut(false),
        style().color(lit),
    )
}

fn menu_description(_: &Theme, _: &Look<MenuItemState>, _base: Style) -> Style {
    style().color(MUTED_FOREGROUND)
}

/// The 16 px box of a check or radio item's `check`, at the row's end (at
/// its start in the menu bar), in the row's color.
fn check(s: &Look<MenuItemState>, look: MenuLook) -> Style {
    let at = style()
        .position(Position::Absolute)
        .top(pct(50.0) - 8.0)
        .size(16.0, 16.0)
        .color(POPOVER_FOREGROUND);
    let at = if look.bar {
        at.left(6.0)
    } else {
        at.right(8.0)
    };
    if s.destructive {
        at
    } else {
        when_lit(s, at, style().color(ACCENT_FOREGROUND))
    }
}

fn menu_check(_: &Theme, s: &Look<MenuItemState>, _base: Style) -> Style {
    check(s, kind(s))
}

/// The submenu's 16 px `chevron-right`, in the row's color.
fn menu_submenu(_: &Theme, s: &Look<MenuItemState>, _base: Style) -> Style {
    when_lit(
        s,
        style()
            .size(16.0, 16.0)
            .flex_shrink(0.0)
            .color(POPOVER_FOREGROUND),
        style().color(ACCENT_FOREGROUND),
    )
}

/// `pl-7` (`pl-8` in the menu bar): the inset box, then the row's gap.
fn menu_inset(_: &Theme, s: &Look<PartState>, _base: Style) -> Style {
    style()
        .width(if kind(s).bar { 16.0 } else { 12.0 })
        .flex_shrink(0.0)
}

/// tooltip.tsx: `inline-flex center gap-1.5 px-3 py-1.5 text-xs` in `bg-foreground`.
fn tooltip_surface(_: &Theme, _: &Look<PopupState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(6.0)
        .max_width(320.0)
        .padding_x(12.0)
        .padding_y(6.0)
        .border_radius(0.0)
        .background(FOREGROUND)
        .color(BACKGROUND)
        .font_size(12.0)
}

/// tooltip.tsx's arrow: a `size-2.5` square in `bg-foreground`, turned 45°,
/// half out of the edge toward the trigger.
fn tooltip_arrow(_: &Theme, s: &Look<TooltipArrowState>, _base: Style) -> Style {
    const SIDE: f64 = 10.0;
    let base = style()
        .position(Position::Absolute)
        .size(SIDE, SIDE)
        .rotate(45.0)
        .background(FOREGROUND);
    let along = s.offset - SIDE / 2.0;
    match s.side {
        // `-bottom-2.5 translate-y-[calc(-50%-2px)]`.
        Edge::Top => base.left(along).top(pct(100.0) - 7.0),
        // `top-1 translate-y-[calc(-50%-2px)]`.
        Edge::Bottom => base.left(along).top(-3.0),
        // `top-1/2! -right-1 -translate-y-1/2`.
        Edge::Left => base.top(pct(50.0) - SIDE / 2.0).left(pct(100.0) - 6.0),
        // `top-1/2! -left-1 -translate-y-1/2`.
        Edge::Right => base.top(pct(50.0) - SIDE / 2.0).left(-4.0),
    }
}

/// hover-card.tsx: `w-64 p-2.5 shadow-md` on the popover surface.
fn hover_card_surface(_: &Theme, _: &Look<HoverCardSurfaceState>, _base: Style) -> Style {
    shadow_md(popover_surface())
        .flex_direction(FlexDirection::Column)
        .line_height(1.625)
        .width(256.0)
        .max_width(pct(100.0))
        .padding(10.0)
}

/// The bar's dropdown surface: `sub` for a submenu. menubar.tsx's widths.
pub(crate) fn bar_surface(sub: bool) -> Style {
    if sub {
        shadow_lg(menu_base()).min_width(BAR.sub_min_width)
    } else {
        shadow_md(menu_base()).min_width(BAR.min_width)
    }
}

/// A row of the menu bar's dropdown.
pub(crate) fn bar_item(t: &Theme, state: MenuItemState) -> Style {
    item(t, &Look::new(state).with_variant(MenuVariant::Bar), BAR)
}

/// The check of a row of the menu bar's dropdown.
pub(crate) fn bar_check(state: MenuItemState) -> Style {
    check(&Look::new(state).with_variant(MenuVariant::Bar), BAR)
}

/// The shortcut of a row of the menu bar's dropdown.
pub(crate) fn bar_shortcut(state: MenuItemState) -> Style {
    let lit = if state.destructive {
        DESTRUCTIVE
    } else {
        ACCENT_FOREGROUND
    };
    when_lit(
        &Look::new(state).with_variant(MenuVariant::Bar),
        crate::recipe_parts::menu_shortcut(false),
        style().color(lit),
    )
}

/// The overlay recipes: select, menu, tooltip and hover card, and the
/// dialog, command palette and combobox recipes.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme
        .with(
            SelectTheme::default()
                .trigger(select_trigger)
                .surface(select_surface)
                .option(select_option)
                .check(select_check)
                .indicator(select_indicator)
                .description(select_description),
        )
        .with(
            MenuTheme::default()
                .trigger(menu_trigger)
                .surface(menu_surface)
                .sub_surface(menu_sub_surface)
                .context_surface(menu_context_surface)
                .item(menu_item)
                .separator(menu_separator)
                .label(menu_label)
                .shortcut(menu_shortcut)
                .description(menu_description)
                .check(menu_check)
                .submenu(menu_submenu)
                .inset(menu_inset),
        )
        .with(
            TooltipTheme::default()
                .surface(tooltip_surface)
                .arrow(tooltip_arrow),
        )
        .with(HoverCardTheme::default().surface(hover_card_surface));
    let theme = crate::recipes_overlays_dialog::install(theme);
    let theme = crate::recipes_overlays_command::install(theme);
    crate::recipes_overlays_combobox::install(theme)
}
