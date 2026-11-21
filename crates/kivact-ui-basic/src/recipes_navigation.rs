//! Role: Lyra recipes of the navigation parts: tabs (the default and the line
//! variant), breadcrumb, navigation menu and pagination.
//!
//! The looks of tabs.tsx, breadcrumb.tsx, navigation-menu.tsx and
//! pagination.tsx. The line variant of the tabs is selected by the tabs'
//! [`TabsVariant`](crate::variants::TabsVariant).

use crate::color::{BACKGROUND, BORDER, FOREGROUND, INPUT, MUTED, MUTED_FOREGROUND};
use crate::recipe_parts::{popover_surface, shadow_md, theme_ring, wide_focus_ring};
use crate::variants::TabsVariant;
use kivact::geometry::Rect;
use kivact::interaction::{NavOpen, TabListState, TabState};
use kivact::style::{
    Align, Color, FlexDirection, Justify, Position, Style, Theme, ThemeMode, pct, style,
};
use kivact::ui::PageItem;
use kivact::ui::theme::{
    BreadcrumbItemState, BreadcrumbTheme, Look, NavigationLinkState, NavigationMenuTheme,
    NavigationTriggerState, PaginationItemState, PaginationTheme, TabsTheme,
};
use kivact::view::{Orientation, ReadingDirection, use_direction};

/// `flex-1 text-xs/relaxed outline-none`: a panel draws no focus ring; the
/// root's `gap-2` is the panel's leading padding.
fn tab_panel(_: &Theme, s: &Look<TabState>, _base: Style) -> Style {
    if s.variant::<TabsVariant>() == Some(TabsVariant::Line) {
        return line_panel(s);
    }
    let base = style()
        .flex_grow(1.0)
        .font_size(12.0)
        .line_height(1.625)
        .color(FOREGROUND)
        .focus_visible(style().outline_none());
    if s.orientation == Orientation::Vertical {
        base.padding_left(8.0)
    } else {
        base.padding_top(8.0)
    }
}

/// The tab list: `bg-muted` pill, a row of `h-8`, or a column when vertical.
fn tabs_list(_: &Theme, s: &Look<TabListState>, _base: Style) -> Style {
    if s.variant::<TabsVariant>() == Some(TabsVariant::Line) {
        return line_list(s);
    }
    let base = style()
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .align_self(Align::Start)
        .padding(3.0)
        .border_radius(0.0)
        .background(MUTED)
        .color(MUTED_FOREGROUND);
    if s.orientation == Orientation::Vertical {
        // `group-data-vertical/tabs:h-fit flex-col`.
        base.flex_direction(FlexDirection::Column)
    } else {
        base.flex_direction(FlexDirection::Row).height(32.0)
    }
}

fn tab(t: &Theme, s: &Look<TabState>, _base: Style) -> Style {
    if s.variant::<TabsVariant>() == Some(TabsVariant::Line) {
        return line_tab(t, s);
    }
    let dark = t.mode() == ThemeMode::Dark;
    let rest = if dark {
        MUTED_FOREGROUND
    } else {
        FOREGROUND.with_alpha(0.6)
    };
    let base = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(6.0)
        .padding_x(6.0)
        .border_radius(0.0)
        .border(1.0, Color::TRANSPARENT)
        .font_size(12.0)
        .font_weight(500)
        .text_wrap(false)
        .color(rest)
        .hover(style().color(FOREGROUND));
    let base = if s.orientation == Orientation::Vertical {
        // `group-data-vertical/tabs:w-full justify-start py-[calc(--spacing(1.25))]`.
        base.width(pct(100.0))
            .justify_content(Justify::Start)
            .padding_y(5.0)
    } else {
        // `h-[calc(100%-1px)] flex-1 py-0.5`.
        base.flex_grow(1.0).height(pct(100.0) - 1.0).padding_y(2.0)
    };
    let base = if s.selected {
        let (fill, border) = if dark {
            (INPUT.with_alpha(0.3), INPUT)
        } else {
            (BACKGROUND, Color::TRANSPARENT)
        };
        base.background(fill).border_color(border).color(FOREGROUND)
    } else {
        base
    };
    // `focus-visible:ring-[3px] ring-ring/50 outline-1 outline-ring`.
    base.focus_visible(wide_focus_ring())
}

/// The line variant's list: a transparent row of `h-8`, `gap-1`, or a column
/// when vertical.
fn line_list(s: &Look<TabListState>) -> Style {
    let base = style()
        .align_self(Align::Start)
        .align_items(Align::Center)
        .gap(4.0)
        .padding(3.0)
        .border_radius(0.0)
        .background(Color::TRANSPARENT)
        .color(MUTED_FOREGROUND);
    if s.orientation == Orientation::Vertical {
        base.flex_direction(FlexDirection::Column)
            .align_items(Align::Stretch)
            .justify_content(Justify::Start)
    } else {
        base.flex_direction(FlexDirection::Row)
            .justify_content(Justify::Center)
            .height(32.0)
    }
}

/// The line variant's tab, with the 2 px bar in the foreground under it while
/// selected: 5 px below its box, or 4 px beside it when vertical.
fn line_tab(t: &Theme, s: &Look<TabState>) -> Style {
    let dark = t.mode() == ThemeMode::Dark;
    let rest = if dark {
        MUTED_FOREGROUND
    } else {
        FOREGROUND.with_alpha(0.6)
    };
    let vertical = s.orientation == Orientation::Vertical;
    let base = style()
        .position(Position::Relative)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(6.0)
        .padding_x(6.0)
        .border_radius(0.0)
        .border(1.0, Color::TRANSPARENT)
        .font_size(12.0)
        .font_weight(500)
        .text_wrap(false)
        .color(if s.selected { FOREGROUND } else { rest })
        .hover(style().color(FOREGROUND))
        // `focus-visible:ring-[3px] ring-ring/50 outline-1 outline-ring`.
        .focus_visible(wide_focus_ring());
    let base = if vertical {
        base.justify_content(Justify::Start).padding_y(5.0)
    } else {
        base.justify_content(Justify::Center)
            .flex_grow(1.0)
            .height(pct(100.0) - 1.0)
            .padding_y(2.0)
    };
    if !s.selected {
        return base;
    }
    let bar = FOREGROUND;
    base.paint(move |d, size| {
        let rect = if vertical {
            Rect::new(size.width + 2.0, 0.0, 2.0, size.height)
        } else {
            Rect::new(0.0, size.height + 3.0, size.width, 2.0)
        };
        d.fill_rect(rect, bar);
    })
}

/// `flex-1 text-xs/relaxed`, after the root's `gap-2`.
fn line_panel(s: &Look<TabState>) -> Style {
    let base = style()
        .flex_grow(1.0)
        .font_size(12.0)
        .line_height(1.625)
        .color(FOREGROUND);
    if s.orientation == Orientation::Vertical {
        base.padding_left(8.0)
    } else {
        base.padding_top(8.0)
    }
}

/// `text-xs`, muted; reversed in a right-to-left reading direction.
fn breadcrumb_root(_: &Theme, _: &Look<()>, _base: Style) -> Style {
    style().font_size(12.0).color(MUTED_FOREGROUND)
}

/// The trail: a wrapping row of `gap-1.5`.
fn breadcrumb_list(_: &Theme, _: &Look<()>, _base: Style) -> Style {
    style()
        .flex_direction(if use_direction() == ReadingDirection::Rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .flex_wrap(true)
        .align_items(Align::Center)
        .gap(6.0)
}

/// A crumb: plain text, muted unless it is the current page.
fn breadcrumb_item(_: &Theme, s: &Look<BreadcrumbItemState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .height(kivact::style::Length::Auto)
        .min_width(0.0)
        .padding(0.0)
        .gap(0.0)
        .border_radius(0.0)
        .border(0.0, Color::TRANSPARENT)
        .background(Color::TRANSPARENT)
        .font_size(12.0)
        .font_weight(400)
        .text_wrap(false)
        .color(if s.is_current() {
            FOREGROUND
        } else {
            MUTED_FOREGROUND
        })
        .hover(style().color(FOREGROUND).background(Color::TRANSPARENT))
}

fn breadcrumb_separator(_: &Theme, _: &Look<()>, _base: Style) -> Style {
    style().flex_shrink(0.0).align_items(Align::Center)
}

/// The row of the menu's triggers, centred.
fn navigation_root(_: &Theme, _: &Look<NavOpen>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(0.0)
}

fn navigation_trigger(t: &Theme, s: &Look<NavigationTriggerState>, _base: Style) -> Style {
    navigation_trigger_style(t, s.open)
}

fn navigation_action(t: &Theme, _: &Look<NavOpen>, _base: Style) -> Style {
    navigation_trigger_style(t, false)
}

/// `h-9 px-4 py-2 text-xs font-medium`: a trigger, muted while its panel is open.
fn navigation_trigger_style(t: &Theme, open: bool) -> Style {
    let hot = style().background(MUTED);
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .flex_shrink(0.0)
        .height(36.0)
        .padding_x(10.0)
        .padding_y(6.0)
        .border_radius(0.0)
        .font_size(12.0)
        .font_weight(500)
        .text_wrap(false)
        .color(FOREGROUND)
        .background(if open {
            MUTED.with_alpha(0.5)
        } else {
            Color::TRANSPARENT
        })
        .hover(hot.clone())
        .focus(hot)
        .focus_visible(theme_ring(t))
}

/// The panel of the menu: a popover surface with a shadow, under the trigger.
fn navigation_panel(_: &Theme, _: &Look<NavOpen>, _base: Style) -> Style {
    shadow_md(popover_surface())
        .flex_direction(FlexDirection::Column)
        .padding(4.0)
        .margin_top(8.0)
}

/// A link of the panel: 8 px around its title and description.
fn navigation_link(t: &Theme, s: &Look<NavigationLinkState>, _base: Style) -> Style {
    let hot = style().background(MUTED);
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(if s.has_description {
            Align::Start
        } else {
            Align::Center
        })
        .gap(8.0)
        .padding(8.0)
        .border_radius(0.0)
        .font_size(12.0)
        .color(FOREGROUND)
        .hover(hot.clone())
        .focus(hot)
        .focus_visible(theme_ring(t))
}

fn navigation_link_title(_: &Theme, _: &Look<NavigationLinkState>, _base: Style) -> Style {
    style().font_weight(500)
}

fn navigation_link_description(_: &Theme, _: &Look<NavigationLinkState>, _base: Style) -> Style {
    style()
        .line_height(1.375)
        .line_clamp(2)
        .color(MUTED_FOREGROUND)
}

fn navigation_link_icon(_: &Theme, _: &Look<NavigationLinkState>, _base: Style) -> Style {
    style().flex_shrink(0.0)
}

/// The row of the pages, centred and reversed in a right-to-left reading direction.
fn pagination_root(_: &Theme, _: &Look<()>, _base: Style) -> Style {
    style()
        .flex_direction(if use_direction() == ReadingDirection::Rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .width(pct(100.0))
        .justify_content(Justify::Center)
        .align_items(Align::Center)
        .gap(2.0)
}

/// A page cell: a 32 px box, outlined and filled when it is the current page.
fn pagination_item(t: &Theme, s: &Look<PaginationItemState>, _base: Style) -> Style {
    let dark = t.mode() == ThemeMode::Dark;
    let fill = if s.current {
        if dark {
            INPUT.with_alpha(0.3)
        } else {
            BACKGROUND
        }
    } else {
        Color::TRANSPARENT
    };
    let hover = if dark { MUTED.with_alpha(0.5) } else { MUTED };
    let base = style()
        .height(32.0)
        .min_width(32.0)
        .border_radius(0.0)
        .font_size(12.0)
        .font_weight(500)
        .color(FOREGROUND)
        .background(fill)
        .border(
            1.0,
            if s.current {
                if dark { INPUT } else { BORDER }
            } else {
                Color::TRANSPARENT
            },
        )
        .hover(style().background(hover));
    match s.item {
        PageItem::Page(_) => base.padding_x(0.0),
        PageItem::Previous => base.padding_left(6.0).padding_right(10.0),
        PageItem::Next => base.padding_left(10.0).padding_right(6.0),
        _ => base.padding_x(10.0),
    }
}

/// The ellipsis of a page run: a 32 px box with its mark centred.
fn pagination_ellipsis(_: &Theme, _: &Look<()>, _base: Style) -> Style {
    style()
        .size(32.0, 32.0)
        .flex_shrink(0.0)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

/// Installs the navigation parts of the kit.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            TabsTheme::default()
                .list(tabs_list)
                .tab(tab)
                .panel(tab_panel),
        )
        .with(
            BreadcrumbTheme::default()
                .root(breadcrumb_root)
                .list(breadcrumb_list)
                .item(breadcrumb_item)
                .separator(breadcrumb_separator),
        )
        .with(
            NavigationMenuTheme::default()
                .root(navigation_root)
                .trigger(navigation_trigger)
                .action(navigation_action)
                .panel(navigation_panel)
                .link(navigation_link)
                .link_title(navigation_link_title)
                .link_description(navigation_link_description)
                .link_icon(navigation_link_icon),
        )
        .with(
            PaginationTheme::default()
                .root(pagination_root)
                .item(pagination_item)
                .ellipsis(pagination_ellipsis),
        )
}
